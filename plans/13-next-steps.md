목표: Gaut 컴파일러를 Gaut로 완성하고 self-host 고정점 검증을 강화하며, 경고/테스트/FFI를 정리한다.

작업 항목
1) 컴파일러 구현 채우기
   - `compiler/ast.gaut`: AST/Type 정의를 Rust 버전과 동일하게 이식.
   - `compiler/parser.gaut`: 토큰화 및 우선순위 파서 구현(함수/레코드/조건/블록/참조/이동/복사).
   - `compiler/typecheck.gaut`: move/escape 규칙, 블록 수명 검사, 타입 일치 검증 구현.
   - `compiler/cgen.gaut`: 현재 Rust cgen 기능(아레나 삽입, Str/Bytes concat, ref/record 필드 접근) 반영.

2) FFI 빌트인/런타임 확장
   - runtime C에 `read_file(path: Str)->Str`, `write_file(path: Str, data: Str)->Unit`, `args()->Bytes` 구현.
   - Gaut std 바인딩 추가, 컴파일러에서 파일 IO/입력 인자 처리에 사용.

3) self_host 루프 보강
   - `SELF_HOST_COMPILER=1 ./scripts/self_host.sh` 경로에서 stage0→stage1→stage2 C 해시 비교를 실패 시 non-zero로 종료.
   - 컴파일러 C 출력 해시를 로그에 명확히 표시하고, 예제와 동일한 결정성 흐름 유지.

4) 경고/정리
   - parser Token 가시성 수정으로 `private_interfaces` 경고 해소.
   - `IndexMap::remove` → `shift_remove` 등으로 deprecation 제거.

5) 테스트 강화
   - cgen golden/단위 테스트에 레코드/참조/Bytes concat 케이스 추가.
   - self_host 스크립트에 확장된 예제와 compiler 루프 스모크 포함(옵션 플래그 존중).
   - 필요한 경우 runtime C에 대한 간단한 단위 테스트 혹은 Rust FFI 스모크 추가.
