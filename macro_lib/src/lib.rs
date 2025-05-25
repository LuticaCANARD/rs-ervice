use proc_macro::TokenStream;
use syn::{ItemStruct,parse_macro_input,ItemImpl};
use quote::quote;
// --- #[r_service_struct] 매크로 ---

// tokio feature가 활성화된 경우
#[proc_macro_attribute]
#[cfg(feature = "tokio")]
pub fn r_service_struct(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // tokio 환경에 맞는 코드 생성 (아직 구현되지 않음)
    // TODO: tokio용 코드 작성
    item
}

// tokio feature가 비활성화된 경우
#[proc_macro_attribute]
#[cfg(not(feature = "tokio"))]
pub fn r_service_struct(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // attr: 애트리뷰트에 전달된 인자 (예: #[r_service_struct(name = "foo")]) -> 현재는 사용 안 함
    // item: 애트리뷰트가 붙은 아이템 (구조체 정의)

    // 1. 입력 파싱
    let input_struct = parse_macro_input!(item as ItemStruct);
    let struct_name = &input_struct.ident; // 구조체 이름 (예: MyService)

    // 2. 코드 생성
    let expanded = quote! {
        #input_struct
        // 추가 코드 생성 가능
    };

    // 3. 생성된 코드 반환
    TokenStream::from(expanded)
}

// --- #[r_service] 매크로 ---

// tokio feature가 활성화된 경우
#[proc_macro_attribute]
#[cfg(feature = "tokio")]
pub fn r_service(attr: TokenStream, item: TokenStream) -> TokenStream {
    // tokio 환경에 맞는 코드 생성 (service 모듈 사용)
    service::r_service(attr, item)
}

// tokio feature가 비활성화된 경우
#[proc_macro_attribute]
#[cfg(not(feature = "tokio"))]
pub fn r_service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_impl = parse_macro_input!(item as ItemImpl);

    let expanded = quote! {
        #input_impl
        // 추가 코드 생성 가능
    };

    TokenStream::from(expanded)
}

