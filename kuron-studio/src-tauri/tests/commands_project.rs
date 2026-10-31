//! Integration test command project/folder/import/move + durabilitas
//! `projects.json` (issue #62). Semua lewat command NYATA via `tauri::test::mock_builder`
//! melalui `common::test_app`.

mod common;

use std::fs::File;
use std::io::Write;
use serde_json::json;

fn obj(v: &serde_json::Value) -> &serde_json::Map<String, serde_json::Value> {
    v.as_object().expect("respons harus objek JSON")
}

/// Kumpulkan semua halaman dari semua folder — respons `Project` bersarang
/// (`folders[].pages[]`), tanpa array `pages` top-level.
fn all_pages(proj: &serde_json::Value) -> Vec<&serde_json::Value> {
    proj["folders"]
        .as_array()
        .expect("folders array")
        .iter()
        .flat_map(|f| f["pages"].as_array().expect("pages array").iter())
        .collect()
}

#[test]
fn project_crud_roundtrip() {
    let (app, dir) = common::test_app();

    // 1. Awal kosong
    let list = common::invoke(&app, "list_projects", json!({})).expect("list awal");
    assert_eq!(list.as_array().expect("list array").len(), 0);

    // 2. Nama kosong ditolak
    let err = common::invoke(&app, "create_project", json!({"name": "  "})).unwrap_err();
    assert!(err.as_str().unwrap().contains("empty"), "err: {err}");

    // 3. Buat dua project -> periksa format camelCase & id unik
    let b = common::invoke(&app, "create_project", json!({"name": "Beta"})).expect("buat Beta");
    let a = common::invoke(&app, "create_project", json!({"name": "Alpha"})).expect("buat Alpha");
    assert!(b.get("id").is_some(), "camelCase id wajib ada");
    assert!(b.get("updatedAt").is_some(), "camelCase updatedAt wajib ada");
    assert!(b.get("folders").is_some(), "camelCase folders wajib ada");

    let id_b = obj(&b)["id"].as_str().unwrap().to_string();
    let id_a = obj(&a)["id"].as_str().unwrap().to_string();
    assert_ne!(id_a, id_b, "id project harus unik");

    // 4. list_projects terurut berdasarkan nama
    let list = common::invoke(&app, "list_projects", json!({})).expect("list isi");
    let names: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["Alpha", "Beta"], "list harus urut nama");

    // 5. get_project happy path & error path
    let got = common::invoke(&app, "get_project", json!({"project_id": id_a})).expect("get");
    assert_eq!(obj(&got)["name"], json!("Alpha"));
    let err = common::invoke(&app, "get_project", json!({"project_id": "tidak-ada"})).unwrap_err();
    assert!(err.as_str().unwrap().contains("project not found"), "err: {err}");

    // 6. delete_project happy path, double delete error, dan list kembali kosong
    common::invoke(&app, "delete_project", json!({"project_id": id_a})).expect("hapus A");
    let err = common::invoke(&app, "delete_project", json!({"project_id": id_a})).unwrap_err();
    assert!(err.as_str().unwrap().contains("project not found"), "err: {err}");

    common::invoke(&app, "delete_project", json!({"project_id": id_b})).expect("hapus B");
    let list_after = common::invoke(&app, "list_projects", json!({})).expect("list akhir");
    assert_eq!(list_after.as_array().unwrap().len(), 0);

    // 7. Side effect disk: projects.json tetap valid
    let pj_path = dir.join("projects.json");
    assert!(pj_path.exists(), "projects.json harus ada di disk");
    let content: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&pj_path).expect("baca projects.json"),
    ).expect("valid json");
    assert!(content.is_object());
}

#[test]
fn folder_crud_subtree_dan_error() {
    let (app, _dir) = common::test_app_tagged("folder-crud");
    let p = common::invoke(&app, "create_project", json!({"name": "F"})).expect("buat project");
    let pid = obj(&p)["id"].as_str().unwrap().to_string();
    let default_folder = obj(&p)["folders"].as_array().unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 1. Nama kosong & duplikat sesaudara ditolak
    let err = common::invoke(
        &app,
        "create_folder",
        json!({"project_id": pid, "name": " ", "parent_id": null}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("empty"), "err: {err}");

    let ch = common::invoke(
        &app,
        "create_folder",
        json!({"project_id": pid, "name": "Ch 1", "parent_id": null}),
    )
    .expect("buat folder");
    let ch_id = obj(&ch)["id"].as_str().unwrap().to_string();

    let err = common::invoke(
        &app,
        "create_folder",
        json!({"project_id": pid, "name": "Ch 1", "parent_id": null}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("sudah ada"), "err: {err}");

    // 2. Parent tak dikenal ditolak
    let err = common::invoke(
        &app,
        "create_folder",
        json!({"project_id": pid, "name": "X", "parent_id": "nope"}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("parent folder not found"), "err: {err}");

    // 3. Anak di bawah Ch 1, lalu rename + rename nama kosong
    let kid = common::invoke(
        &app,
        "create_folder",
        json!({"project_id": pid, "name": "Sub 1", "parent_id": ch_id}),
    )
    .expect("buat anak");
    let kid_id = obj(&kid)["id"].as_str().unwrap().to_string();

    let renamed = common::invoke(
        &app,
        "rename_folder",
        json!({"project_id": pid, "folder_id": ch_id, "name": "Ch 01"}),
    )
    .expect("rename");
    assert_eq!(obj(&renamed)["name"], json!("Ch 01"));

    let err = common::invoke(
        &app,
        "rename_folder",
        json!({"project_id": pid, "folder_id": kid_id, "name": "  "}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("empty"), "err: {err}");

    // 4. Hapus subtree Ch 01 -> anak ikut hilang
    common::invoke(
        &app,
        "delete_folder",
        json!({"project_id": pid, "folder_id": ch_id}),
    )
    .expect("hapus subtree");

    let got = common::invoke(&app, "get_project", json!({"project_id": pid})).expect("get");
    let folders = obj(&got)["folders"].as_array().unwrap();
    assert_eq!(folders.len(), 1, "tinggal folder default");
    assert!(
        !folders.iter().any(|f| f["id"] == json!(kid_id)),
        "anak subtree ikut terhapus"
    );

    // 5. Folder terakhir tidak boleh dihapus; folder asing ditolak.
    // Guard "terakhir" jalan duluan di src, jadi uji id asing butuh
    // folder kedua agar tak tertutup guard tersebut.
    let err = common::invoke(
        &app,
        "delete_folder",
        json!({"project_id": pid, "folder_id": default_folder}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("terakhir"), "err: {err}");

    common::invoke(
        &app,
        "create_folder",
        json!({"project_id": pid, "name": "Extra", "parent_id": null}),
    )
    .expect("folder kedua");

    let err = common::invoke(
        &app,
        "delete_folder",
        json!({"project_id": pid, "folder_id": "asing"}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("folder not found"), "err: {err}");
}

#[test]
fn import_pages_folder_campur_dan_zip() {
    let (app, dir) = common::test_app_tagged("import-campur");
    let p = common::invoke(&app, "create_project", json!({"name": "ImportTest"})).expect("buat project");
    let pid = obj(&p)["id"].as_str().unwrap().to_string();

    let ch = common::invoke(
        &app,
        "create_folder",
        json!({"project_id": pid, "name": "Ch 1", "parent_id": null}),
    )
    .expect("buat folder Ch 1");
    let ch_id = obj(&ch)["id"].as_str().unwrap().to_string();

    // Siapkan 2 gambar PNG mentah
    let src_dir = dir.join("fixtures");
    std::fs::create_dir_all(&src_dir).unwrap();
    let img1 = src_dir.join("01.png");
    let img2 = src_dir.join("02.png");
    common::write_png(&img1, 100, 150);
    common::write_png(&img2, 120, 160);

    // Siapkan 1 file ZIP berisi 1 gambar PNG
    let zip_path = src_dir.join("bundle.zip");
    {
        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("03.png", options).unwrap();
        let mut png_bytes = Vec::new();
        let img3 = image::RgbImage::from_pixel(80, 100, image::Rgb([10, 20, 30]));
        image::DynamicImage::ImageRgb8(img3)
            .write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .unwrap();
        zip.write_all(&png_bytes).unwrap();
        zip.finish().unwrap();
    }

    // Panggil command import_pages nyata
    let res = common::invoke(
        &app,
        "import_pages",
        json!({
            "project_id": pid,
            "paths": [
                img1.to_string_lossy(),
                img2.to_string_lossy(),
                zip_path.to_string_lossy(),
            ],
            "folder_id": ch_id,
        }),
    )
    .expect("import_pages sukses");

    let pages = res["pages"].as_array().expect("pages array");
    assert_eq!(pages.len(), 3, "3 halaman berhasil diimpor");
    // Folder tujuan dilaporkan sekali di top-level respons (`ImportResult`).
    assert_eq!(res["folderId"], json!(ch_id));

    // Periksa camelCase fields pada page (`Page`: id/path/status/...)
    let p0 = &pages[0];
    assert!(p0.get("id").is_some());
    assert!(p0.get("path").is_some());
    assert!(p0.get("status").is_some());

    // Periksa side effect di disk: file gambar disalin ke dalam store
    let proj = common::invoke(&app, "get_project", json!({"project_id": pid})).expect("get project");
    let proj_pages = all_pages(&proj);
    assert_eq!(proj_pages.len(), 3);
    for page in proj_pages {
        let stored_path = page["path"].as_str().unwrap();
        assert!(std::path::Path::new(stored_path).exists(), "file gambar harus ada di disk: {stored_path}");
    }
}

#[test]
fn move_pages_parsial_dan_noop() {
    let (app, dir) = common::test_app_tagged("move-pages");
    let p = common::invoke(&app, "create_project", json!({"name": "MoveTest"})).expect("buat project");
    let pid = obj(&p)["id"].as_str().unwrap().to_string();

    let f_a = common::invoke(
        &app,
        "create_folder",
        json!({"project_id": pid, "name": "Folder A", "parent_id": null}),
    )
    .expect("folder A");
    let fa_id = obj(&f_a)["id"].as_str().unwrap().to_string();

    let f_b = common::invoke(
        &app,
        "create_folder",
        json!({"project_id": pid, "name": "Folder B", "parent_id": null}),
    )
    .expect("folder B");
    let fb_id = obj(&f_b)["id"].as_str().unwrap().to_string();

    // Impor 2 halaman ke folder A
    let img_dir = dir.join("move_src");
    std::fs::create_dir_all(&img_dir).unwrap();
    let p1 = img_dir.join("p1.png");
    let p2 = img_dir.join("p2.png");
    common::write_png(&p1, 50, 50);
    common::write_png(&p2, 60, 60);

    let imported = common::invoke(
        &app,
        "import_pages",
        json!({
            "project_id": pid,
            "paths": [p1.to_string_lossy(), p2.to_string_lossy()],
            "folder_id": fa_id,
        }),
    )
    .expect("impor 2 halaman");
    let page_ids: Vec<String> = imported["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap().to_string())
        .collect();

    // 1. Pindah page 1 ke folder B -> moved=1, skipped=0
    let m1 = common::invoke(
        &app,
        "move_pages",
        json!({
            "project_id": pid,
            "page_ids": [page_ids[0].clone()],
            "folder_id": fb_id,
        }),
    )
    .expect("move 1 page");
    assert_eq!(m1["moved"], json!(1));
    assert_eq!(m1["skipped"], json!(0));

    // 2. Pindah lagi page 1 ke folder B: tak ada yang bergerak → src
    // mengembalikan Err ("semua halaman sudah di folder tujuan"), bukan
    // Ok(moved=0) — tanpa efek samping pada store.
    let err = common::invoke(
        &app,
        "move_pages",
        json!({
            "project_id": pid,
            "page_ids": [page_ids[0].clone()],
            "folder_id": fb_id,
        }),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("sudah di folder tujuan"), "err: {err}");

    // 3. Pindah ke folder yang tidak ada -> error
    let err = common::invoke(
        &app,
        "move_pages",
        json!({
            "project_id": pid,
            "page_ids": [page_ids[1].clone()],
            "folder_id": "tidak-ada",
        }),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("folder not found"), "err: {err}");
}

/// `clean_pages`: halaman yang file-nya hilang dari disk dibuang dari store;
/// project tak dikenal ditolak.
#[test]
fn clean_pages_membersihkan_file_hilang() {
    let (app, dir) = common::test_app_tagged("clean-pages");
    let p = common::invoke(&app, "create_project", json!({"name": "CleanMe"})).expect("buat project");
    let pid = obj(&p)["id"].as_str().unwrap().to_string();

    // Impor 1 halaman lalu hapus file fisiknya dari store.
    let src_dir = dir.join("clean_src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let img = src_dir.join("hilang.png");
    common::write_png(&img, 40, 40);
    common::invoke(
        &app,
        "import_pages",
        json!({"project_id": pid, "paths": [img.to_string_lossy()], "folder_id": null}),
    )
    .expect("impor 1 halaman");

    let proj = common::invoke(&app, "get_project", json!({"project_id": pid})).expect("get project");
    let stored = all_pages(&proj)[0]["path"].as_str().unwrap().to_string();
    assert!(std::path::Path::new(&stored).exists(), "file store ada sebelum dihapus");
    std::fs::remove_file(&stored).expect("hapus file fisik halaman");

    let out = common::invoke(
        &app,
        "clean_pages",
        json!({"project_id": pid, "folder_id": null}),
    )
    .expect("clean_pages berhasil");
    assert_eq!(out["removed"], json!(1));
    assert_eq!(out["kept"], json!(0));

    let proj2 = common::invoke(&app, "get_project", json!({"project_id": pid})).expect("get ulang");
    assert_eq!(all_pages(&proj2).len(), 0, "halaman hilang ikut dibuang");

    // Project tak dikenal ditolak.
    let err = common::invoke(
        &app,
        "clean_pages",
        json!({"project_id": "tidak-ada", "folder_id": null}),
    )
    .unwrap_err();
    assert!(err.as_str().unwrap().contains("not found"), "err: {err}");
}

/// `list_models` via mock HTTP bersama: provider custom menunjuk ke mock
/// server, daftar model OpenAI-shape terparse lewat command nyata. (Dari
/// target ini untuk memakai helper mock di semua target issue #62.)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_models_mock_setelah_save_provider() {
    let (app, _dir) = common::test_app_tagged("list-models");
    const MODELS: &str = r#"{"data":[{"id":"mock-gpt-4o"},{"id":"mock-mini"}]}"#;
    let mock_base = common::spawn_mock(MODELS, 2);

    let prov = common::invoke(
        &app,
        "save_provider",
        json!({"input": {
            "providerType": "custom", "name": "Mock List",
            "baseUrl": mock_base, "apiKey": "k", "model": "mock-gpt-4o",
        }}),
    )
    .expect("simpan provider mock");
    let prov_id = prov["id"].as_str().unwrap().to_string();

    let models = common::invoke(&app, "list_models", json!({"provider_id": prov_id}))
        .expect("list_models berhasil");
    let ids: Vec<&str> = models
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["mock-gpt-4o", "mock-mini"]);

    // Provider tak dikenal ditolak.
    let err = common::invoke(&app, "list_models", json!({"provider_id": "tidak-ada"})).unwrap_err();
    assert!(err.as_str().unwrap().contains("not found"), "err: {err}");
}
