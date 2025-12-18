목표: Gaut로 작성된 컴파일러를 실제로 동작하도록 채우고, self-host 루프를 엄격하게 검증한다.

작업 항목
1) 컴파일러 실구현
   - `compiler/ast.gaut`: Rust AST/Type 구조를 이식.
   - `compiler/parser.gaut`: 토큰화 + 우선순위 파서 구현(함수/레코드/if/블록/참조/이동/복사).
   - `compiler/typecheck.gaut`: move/escape, 블록 수명 검사, 타입 일치 규칙 구현.
   - `compiler/cgen.gaut`: 현 C 백엔드 기능(아레나 삽입, Str/Bytes concat, ref/record 접근) 반영해 실제 C 생성.
   - `compiler/main.gaut`: `--emit-c` CLI 처리, 입력 파일 로드, C 파일 쓰기 구현.

2) self_host 루프 엄격화
   - gautc1이 실제 C를 생성하면 `SELF_HOST_COMPILER_STRICT=1`에서 stage0→stage1→stage2 해시 불일치 시 실패하게 전환.
   - 컴파일러 자체 결정성 확보(선언/필드/맵 순회 정렬).

3) 런타임/FFI 보완
   - `args()`를 실제 argv 기반으로 채우고, Bytes→Str 유틸(std) 추가.
   - `read_file`/`write_file` 반환 규약과 오류 처리 정리(성공/실패 신호).

4) 테스트/경고 정리
   - 컴파일러 경로 스모크 테스트 추가(간단 .gaut → C 비교).
   - parser/token visibility 등 경고 재검증.
   - self_host 스크립트는 해시 로그를 요약 출력하고, 엄격 모드에서만 실패 처리.

검증 기준
- `SELF_HOST_COMPILER_STRICT=1 ./scripts/self_host.sh`가 stage0→stage1→stage2 해시를 비교하고 불일치 시 실패.
- gautc1/gautc2가 실제 C를 생성하며, 예제와 컴파일러 소스에 대해 결정성 확보.
- argv 기반 `args()`가 동작하고, `read_file`/`write_file` 규약이 문서/구현/테스트에 반영.
