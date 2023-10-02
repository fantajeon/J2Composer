use jintemplify::function::register_functions;
use tera::{Context, Tera};

#[test]
fn test_read_file_function() {
    let mut tera = Tera::default();
    register_functions(&mut tera);

    // 경로 설정
    let file_path = "tests/fixtures/function_sample.txt";

    // Tera 컨텍스트 설정
    let mut context = Context::new();
    context.insert("file_path", &file_path);

    // 템플릿에서 함수 사용
    let template_str = r#"{{ read_file(file_path=file_path) }}"#;
    let rendered = tera.render_str(template_str, &context).unwrap();

    // sample_file.txt의 내용이 "Hello, World!"라고 가정
    assert_eq!(rendered, "Hello, World!");
}
