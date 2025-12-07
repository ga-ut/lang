목표: C 트랜스파일러 스켈레톤.

선행: 05-interpreter.md 완료.

산출물
- crates/cgen/src/lib.rs

작업
- AST→C 코드: 함수/변수/반환 매핑
- 각 함수/블록에 로컬 아레나 생성·리셋 코드 삽입
- 기본 타입/레코드/참조를 C struct로 정의

테스트
- 단일 파일 트랜스파일 → 생성된 C 코드를 clang 빌드(실행은 선택)
- 빌드 스크립트 작성(예: scripts/build_c.sh)
