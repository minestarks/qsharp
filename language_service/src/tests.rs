// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod test_fs;

use crate::{
    protocol::{DiagnosticUpdate, ErrorKind},
    Encoding, LanguageService, UpdateWorker,
};
use expect_test::{expect, Expect};
use futures::Future;
use qsc::{
    compile::{self},
    line_column::Position,
    project,
};
use std::{cell::RefCell, rc::Rc};
use test_fs::{dir, file, FsNode, TestProjectHost};

#[allow(clippy::await_holding_refcell_ref)]
#[tokio::test]
async fn single_document() {
    run_async_ls_test(|ls, received_errors| async move {
        let mut ls = ls.borrow_mut();

        ls.update_document("foo.qs", 1, "namespace Foo { }");

        ls.pending_updates().wait().await;

        check_errors_and_compilation(
            &ls,
            &mut received_errors.borrow_mut(),
            "foo.qs",
            &(expect![[r#"
                []
            "#]]),
            &(expect![[r#"
            SourceMap {
                sources: [
                    Source {
                        name: "foo.qs",
                        contents: "namespace Foo { }",
                        offset: 0,
                    },
                ],
                common_prefix: None,
                entry: None,
            }
        "#]]),
        );

        ls.stop_updates();
    })
    .await;
}

#[allow(clippy::await_holding_refcell_ref)]
#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn single_document_update() {
    run_async_ls_test(|ls, received_errors| async move {
        let mut ls = ls.borrow_mut();

        ls.update_document("foo.qs", 1, "namespace Foo { }");

        ls.pending_updates().wait().await;

        check_errors_and_compilation(
            &ls,
            &mut received_errors.borrow_mut(),
            "foo.qs",
            &(expect![[r#"
                []
            "#]]),
            &(expect![[r#"
            SourceMap {
                sources: [
                    Source {
                        name: "foo.qs",
                        contents: "namespace Foo { }",
                        offset: 0,
                    },
                ],
                common_prefix: None,
                entry: None,
            }
        "#]]),
        );

        // UPDATE 2
        ls.update_document(
            "foo.qs",
            1,
            "namespace Foo { @EntryPoint() operation Bar() : Unit {} }",
        );

        ls.pending_updates().wait().await;

        check_errors_and_compilation(
            &ls,
            &mut received_errors.borrow_mut(),
            "foo.qs",
            &(expect![[r#"
                []
            "#]]),
            &(expect![[r#"
            SourceMap {
                sources: [
                    Source {
                        name: "foo.qs",
                        contents: "namespace Foo { @EntryPoint() operation Bar() : Unit {} }",
                        offset: 0,
                    },
                ],
                common_prefix: None,
                entry: None,
            }
        "#]]),
        );

        ls.stop_updates();
    })
    .await;
}

#[allow(clippy::await_holding_refcell_ref)]
#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn document_in_project() {
    println!("document_in_project : start");
    run_async_ls_test(|ls, received_errors| async move {
        let mut ls = ls.borrow_mut();
        println!("document_in_project : task start");
        ls.update_document("project/src/this_file.qs", 1, "namespace Foo { }");

        println!("document_in_project : updated document");

        check_errors_and_no_compilation(
            &ls,
            &mut received_errors.borrow_mut(),
            "project/src/this_file.qs",
            &(expect![[r#"
            []
        "#]]),
        );
        println!("document_in_project : checked errors");

        // now process background work
        ls.pending_updates().wait().await;

        println!("document_in_project : drained queue");

        check_errors_and_compilation(
            &ls,
            &mut received_errors.borrow_mut(),
            "project/src/this_file.qs",
            &expect![[r#"
                []
            "#]],
            &expect![[r#"
            SourceMap {
                sources: [
                    Source {
                        name: "project/src/other_file.qs",
                        contents: "namespace OtherFile { operation Other() : Unit {} }",
                        offset: 0,
                    },
                    Source {
                        name: "project/src/this_file.qs",
                        contents: "namespace Foo { }",
                        offset: 52,
                    },
                ],
                common_prefix: Some(
                    "project/src/",
                ),
                entry: None,
            }
        "#]],
        );

        ls.stop_updates();
    })
    .await;
    println!("document_in_project : end");
}

// the below tests test the asynchronous behavior of the language service.
// we use `get_completions` as a rough analog for all document operations, as
// they all go through the same `document_op` infrastructure.
#[tokio::test]
async fn completions_requested_before_document_load() {
    run_async_ls_test(|ls, _| async move {
        let mut ls = ls.borrow_mut();

        ls.update_document(
            "foo.qs",
            1,
            "namespace Foo { open Microsoft.Quantum.Diagnostics; @EntryPoint() operation Main() : Unit { DumpMachine() } }",
        );

        // we intentionally don't await work to test how LSP features function when
        // a document hasn't fully loaded

        // this should be empty, because the doc hasn't loaded
        assert!(ls
            .get_completions(
                "foo.qs",
                Position {
                    line: 0,
                    column: 76
                }
            )
            .items
            .is_empty());
        ls.stop_updates();
    }).await;
}

#[allow(clippy::await_holding_refcell_ref)]
#[tokio::test]
async fn completions_requested_after_document_load() {
    run_async_ls_test(|ls, _| async move {
        let mut ls = ls.borrow_mut();
        println!("do_test : start");

        // this test is a contrast to `completions_requested_before_document_load`
        // we want to ensure that completions load when the pending updates have been awaited
        ls.update_document(
            "foo.qs",
            1,
            "namespace Foo { open Microsoft.Quantum.Diagnostics; @EntryPoint() operation Main() : Unit { DumpMachine() } }",
        );

        println!("do_test : updated document");

        ls.pending_updates().wait().await;

        println!("do_test : drained queue");

        assert!(&ls
            .get_completions(
                "foo.qs",
                Position {
                    line: 0,
                    column: 92,
                },
            )
            .items
            .iter()
            .any(|item| item.label == "DumpMachine"));

        println!("do_test : end");

        ls.stop_updates();
    })
    .await;
}

async fn run_async_ls_test<F>(
    mut test: impl FnMut(Rc<RefCell<LanguageService>>, Rc<RefCell<Vec<ErrorInfo>>>) -> F,
) where
    F: Future<Output = ()> + 'static,
{
    println!("run_async_ls_test : start");
    let received_errors = Rc::new(RefCell::new(Vec::new()));
    let ls = Rc::new(RefCell::new(LanguageService::new(Encoding::Utf8)));
    let set = tokio::task::LocalSet::new();
    set.run_until(async move {
        println!("run_async_ls_test : spawning tasks");
        // `spawn_local` ensures that the future is spawned on the local
        // task set.
        tokio::try_join!(
            tokio::task::spawn_local(background_work(ls.clone(), received_errors.clone())),
            tokio::task::spawn_local(test(ls.clone(), received_errors.clone()))
        )
        .expect("tasks should not fail");
    })
    .await;
}

fn check_errors_and_compilation(
    ls: &LanguageService,
    received_errors: &mut Vec<ErrorInfo>,
    uri: &str,
    expected_errors: &Expect,
    expected_compilation: &Expect,
) {
    expected_errors.assert_debug_eq(received_errors);
    assert_compilation(ls, uri, expected_compilation);
    received_errors.clear();
}

fn check_errors_and_no_compilation(
    ls: &LanguageService,
    received_errors: &mut Vec<ErrorInfo>,
    uri: &str,
    expected_errors: &Expect,
) {
    expected_errors.assert_debug_eq(received_errors);
    received_errors.clear();

    let state = ls.state.try_borrow().expect("borrow should succeed");
    assert!(state.get_compilation(uri).is_none());
}

fn assert_compilation(ls: &LanguageService, uri: &str, expected: &Expect) {
    let state = ls.state.try_borrow().expect("borrow should succeed");
    let compilation = state
        .get_compilation(uri)
        .expect("compilation should exist");
    expected.assert_debug_eq(&compilation.user_unit().sources);
}

type ErrorInfo = (
    String,
    Option<u32>,
    Vec<compile::ErrorKind>,
    Vec<project::Error>,
);

async fn background_work(
    ls: Rc<RefCell<LanguageService>>,
    received_errors: Rc<RefCell<Vec<ErrorInfo>>>,
) {
    println!("work_ : start");
    let mut worker = create_update_worker(&mut ls.borrow_mut(), &received_errors);
    println!("work_ : created updated worker");
    worker.run().await;
    println!("work_ : work done");
}

fn create_update_worker<'a>(
    ls: &mut LanguageService,
    received_errors: &'a RefCell<Vec<ErrorInfo>>,
) -> UpdateWorker<'a> {
    let worker = ls.create_update_worker(
        |update: DiagnosticUpdate| {
            let project_errors = update.errors.iter().filter_map(|error| match error {
                ErrorKind::Project(error) => Some(error.clone()),
                ErrorKind::Compile(_) => None,
            });
            let compile_errors = update.errors.iter().filter_map(|error| match error {
                ErrorKind::Compile(error) => Some(error.error().clone()),
                ErrorKind::Project(_) => None,
            });

            let mut v = received_errors.borrow_mut();

            v.push((
                update.uri,
                update.version,
                compile_errors.collect(),
                project_errors.collect(),
            ));
        },
        TestProjectHost {
            fs: TEST_FS.with(Clone::clone),
        },
    );
    worker
}

thread_local! { static TEST_FS: Rc<RefCell<FsNode>> = Rc::new(RefCell::new(test_fs())) }

fn test_fs() -> FsNode {
    FsNode::Dir(
        [dir(
            "project",
            [
                file("qsharp.json", "{}"),
                dir(
                    "src",
                    [
                        file(
                            "other_file.qs",
                            "namespace OtherFile { operation Other() : Unit {} }",
                        ),
                        file("this_file.qs", "namespace Foo { }"),
                    ],
                ),
            ],
        )]
        .into_iter()
        .collect(),
    )
}
