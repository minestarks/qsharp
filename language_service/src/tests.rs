// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use crate::{protocol::DiagnosticUpdate, Encoding, JSFileEntry, LanguageService, UpdateWorker};
use expect_test::{expect, Expect};
use futures::Future;
use qsc::{
    compile::{self, ErrorKind},
    line_column::Position,
};
use qsc_project::{EntryType, Manifest, ManifestDescriptor};
use std::{cell::RefCell, future::ready, rc::Rc, sync::Arc};

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
            [
                (
                    "foo.qs",
                    Some(
                        1,
                    ),
                    [
                        Pass(
                            EntryPoint(
                                NotFound,
                            ),
                        ),
                    ],
                ),
            ]
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
            [
                (
                    "foo.qs",
                    Some(
                        1,
                    ),
                    [
                        Pass(
                            EntryPoint(
                                NotFound,
                            ),
                        ),
                    ],
                ),
            ]
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
            [
                (
                    "foo.qs",
                    Some(
                        1,
                    ),
                    [],
                ),
            ]
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
        ls.update_document("this_file.qs", 1, "namespace Foo { }");

        println!("document_in_project : updated document");

        check_errors_and_no_compilation(
            &ls,
            &mut received_errors.borrow_mut(),
            "this_file.qs",
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
            "this_file.qs",
            &expect![[r#"
            [
                (
                    "./qsharp.json",
                    None,
                    [
                        Pass(
                            EntryPoint(
                                NotFound,
                            ),
                        ),
                    ],
                ),
            ]
        "#]],
            &expect![[r#"
            SourceMap {
                sources: [
                    Source {
                        name: "other_file.qs",
                        contents: "namespace OtherFile { operation Other() : Unit {} }",
                        offset: 0,
                    },
                    Source {
                        name: "this_file.qs",
                        contents: "namespace Foo { }",
                        offset: 52,
                    },
                ],
                common_prefix: None,
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

        // this should be empty, because the doc hasn't loaded
        assert_eq!(
            ls.get_completions(
                "foo.qs",
                Position {
                    line: 0,
                    column: 76
                }
            )
            .items
            .len(),
            13
        );

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
    received_errors: &mut Vec<(String, Option<u32>, Vec<ErrorKind>)>,
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
    received_errors: &mut Vec<(String, Option<u32>, Vec<ErrorKind>)>,
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

type ErrorInfo = (String, Option<u32>, Vec<compile::ErrorKind>);

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
            let mut v = received_errors.borrow_mut();

            v.push((
                update.uri.to_string(),
                update.version,
                update
                    .errors
                    .iter()
                    .map(|e| e.error().clone())
                    .collect::<Vec<_>>(),
            ));
        },
        |file| {
            Box::pin(async {
                tokio::spawn(ready(match file.as_str() {
                    "other_file.qs" => (
                        Arc::from(file),
                        Arc::from("namespace OtherFile { operation Other() : Unit {} }"),
                    ),
                    "this_file.qs" => (Arc::from(file), Arc::from("namespace Foo { }")),
                    _ => panic!("unknown file"),
                }))
                .await
                .expect("spawn should not fail")
            })
        },
        |dir_name| {
            Box::pin(async move {
                tokio::spawn(ready(vec![
                    JSFileEntry {
                        name: "src".into(),
                        r#type: (if dir_name.as_str() == "src" {
                            EntryType::File
                        } else {
                            EntryType::Folder
                        }),
                    },
                    JSFileEntry {
                        name: "other_file.qs".into(),
                        r#type: EntryType::File,
                    },
                    JSFileEntry {
                        name: "this_file.qs".into(),
                        r#type: EntryType::File,
                    },
                ]))
                .await
                .expect("spawn should not fail")
            })
        },
        |file| {
            Box::pin(async move {
                tokio::spawn(ready(match file.as_str() {
                    "other_file.qs" | "this_file.qs" => Some(ManifestDescriptor {
                        manifest: Manifest::default(),
                        manifest_dir: ".".into(),
                    }),
                    "foo.qs" => None,
                    _ => panic!("unknown file"),
                }))
                .await
                .expect("spawn should not fail")
            })
        },
    );
    worker
}
