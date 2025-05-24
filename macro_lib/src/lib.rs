use proc_macro::TokenStream;


#[proc_macro_attribute]
pub fn singleton_service(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        // If the attribute has arguments, we return an error.
        return syn::Error::new_spanned(
            quote::quote!(),
            "This attribute takes no arguments"
        ).to_compile_error().into();
    }

    // 2. 아이템 파싱 (이 경우 함수)
    let input_fn = syn::parse_macro_input!(item as syn::ItemFn);

    // 3. 필요한 정보 추출
    let vis = &input_fn.vis;         // 함수의 가시성 (pub 등)
    let sig = &input_fn.sig;         // 함수의 시그니처 (이름, 인자, 반환 타입 등)
    let original_block = &input_fn.block; // 함수의 원래 본문 (블록)
    let fn_name_str = sig.ident.to_string(); // 함수 이름을 문자열로

    // 다른 애트리뷰트들도 그대로 유지
    let other_attrs = &input_fn.attrs;

    // 4. 새로운 코드 생성
    let expanded = quote::quote! {
        #(#other_attrs)* // 원래 함수에 붙어있던 다른 애트리뷰트들을 다시 적용
        #vis #sig { // 원래 함수의 시그니처 사용
            // 함수 시작 로그
            println!("Entering function: {}", #fn_name_str);

            // 원래 함수 본문 실행
            let result = { #original_block };

            // 함수 종료 로그
            println!("Exiting function: {}", #fn_name_str);

            // 원래 함수의 결과 반환
            result
        }
    };

    // 5. 생성된 TokenStream 반환
    TokenStream::from(expanded)
}