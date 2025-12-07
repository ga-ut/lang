목표: Rust 부트스트랩 런타임 아레나와 AST 스켈레톤 구축.

선행: 01-spec.md 완료.

산출물
- crates/runtime/src/arena.rs: 블록/함수 단위 아레나, cap/offset, reset. unsafe 금지.
- crates/frontend/src/ast.rs: Expr/Stmt/Type/Record 정의.
- crates/frontend/src/parser.rs 골격(토큰 정의 등).

작업
- 아레나 cap 초과 시 패닉 처리 추가
- 단위테스트: cap 초과, reset 후 재사용
- AST 구조체: 위치 정보(optional), 기본 타입/레코드/참조/함수 시그니처

완료 조건
- 테스트 통과(cargo test) 기준으로 아레나 동작 확인
- AST 타입이 스펙과 맞음
