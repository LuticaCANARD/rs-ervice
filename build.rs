// build.rs
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::{self, Visit}; // AST 노드 방문을 위해 Visit 트레이트 사용
use syn::{File, Item, Attribute, Ident, ItemStruct};
use quote::quote;

// #[r_service_struct] 애트리뷰트를 가졌는지 확인하는 함수
fn has_r_service_struct_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        // path.is_ident(...)는 단일 식별자 경로만 확인합니다 (예: #[r_service_struct])
        // 만약 애트리뷰트가 crate_name::r_service_struct 처럼 경로를 포함하면,
        // attr.path().segments.last().unwrap().ident == "r_service_struct" 와 같이
        // 더 구체적인 경로 비교가 필요할 수 있습니다.
        // 여기서는 간단히 is_ident로 처리합니다.
        attr.path().is_ident("r_service_struct")
    })
}

// AST를 방문하여 #[r_service_struct]를 가진 구조체 이름을 수집하는 Visitor
#[derive(Default)]
struct ServiceStructVisitor {
    discovered_services: Vec<ServiceInfo>,
}

struct ServiceInfo {
    name: String,
    file_path_str: String, // 파일 경로를 문자열로 저장
}

impl<'ast> Visit<'ast> for ServiceStructVisitor {
    fn visit_item_struct(&mut self, item_struct: &'ast ItemStruct) {
        if has_r_service_struct_attr(&item_struct.attrs) {
            // 파일 경로는 이 Visitor 외부에서 이미 알고 있어야 합니다.
            // 여기서는 ServiceInfo에 name만 채우고, file_path는 외부에서 설정한다고 가정합니다.
            // 실제로는 이 Visitor를 호출하는 쪽에서 파일 경로를 전달받아 ServiceInfo를 완성해야 합니다.
            // 이 예제에서는 main 함수에서 파일 경로와 함께 ServiceInfo를 만듭니다.
            // 여기서는 구조체 이름만 수집하는 데 집중합니다.
            // (아래 main 함수에서 ServiceInfo 생성 로직을 보세요)
        }
        // 기본적으로 모든 하위 노드를 계속 방문합니다.
        visit::visit_item_struct(self, item_struct);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // src 디렉토리 내의 파일이 변경될 때마다 build.rs를 다시 실행
    println!("cargo:rerun-if-changed=src");

    let mut collected_services_info = Vec::<ServiceInfo>::new();
    let src_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("src");

    for entry in glob::glob(&format!("{}/**/*.rs", src_dir.display()))? {
        let path = entry?;
        if path.is_file() {
            let relative_path_str = path.strip_prefix(&src_dir)
                                        .unwrap_or(&path) // src 바깥이면 원래 경로 사용 (이런 경우는 거의 없음)
                                        .to_string_lossy()
                                        .into_owned();

            let content = fs::read_to_string(&path)?;
            let ast = syn::parse_file(&content)?;

            // 각 파일 AST에서 #[r_service_struct] 구조체 찾기
            for item in ast.items {
                if let Item::Struct(item_struct) = item {
                    if has_r_service_struct_attr(&item_struct.attrs) {
                        collected_services_info.push(ServiceInfo {
                            name: item_struct.ident.to_string(),
                            file_path_str: relative_path_str.clone(),
                        });
                    }
                }
            }
        }
    }

    // 수집된 서비스 정보를 사용하여 Rust 코드 토큰 스트림 생성
    let service_info_struct_definition = quote! {
        /// 컴파일 시점에 코드베이스에서 발견된 서비스 타입에 대한 정보입니다.
        /// 이 목록에 포함된 서비스는 `RSContextBuilder`에 등록될 수 있는 후보들입니다.
        /// 특정 `RSContext` 인스턴스에서 실제로 사용 가능한지 여부는
        /// 해당 컨텍스트가 빌드될 때 어떤 서비스들이 명시적으로 `register` 되었는지에 따라 결정됩니다.
        #[derive(Debug, Clone, Copy)]
        pub struct RsServiceDiscoveredInfo {
            /// 서비스 구조체의 이름입니다.
            pub name: &'static str,
            /// 서비스가 정의된 파일의 상대 경로입니다 (src 디렉토리 기준).
            /// 주의: 이는 Rust 모듈 경로가 아니며, 파일 시스템 경로입니다.
            pub file_path: &'static str,
        }
    };

    let service_entries = collected_services_info.iter().map(|info| {
        let name_str = &info.name;
        let file_path_str = &info.file_path_str;
        quote! {
            RsServiceDiscoveredInfo { name: #name_str, file_path: #file_path_str }
        }
    });

    let generated_manifest_code = quote! {
        #service_info_struct_definition

        /// 코드베이스 내에 `#[r_service_struct]`로 정의된 모든 잠재적 서비스들의 명단입니다.
        /// 실제 특정 `RSContext`에서 호출 가능한 서비스 목록은 해당 컨텍스트의
        /// 런타임 introspection 메소드 (예: `RSContext::get_metadata()`)를 통해 확인해야 합니다.
        pub const RS_DISCOVERED_SERVICES: &[RsServiceDiscoveredInfo] = &[
            #( #service_entries ),*
        ];
    };

    // 생성된 코드를 OUT_DIR에 파일로 작성
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let dest_path = out_dir.join("rs_service_generated_manifest.rs");
    fs::write(&dest_path, generated_manifest_code.to_string())?;

    Ok(())
}