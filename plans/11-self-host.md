목표: Gaut를 self-host 가능한 상태로 올려 "언어로 언어를 빌드" 단계까지 확보한다.

선행: 06-cgen, 07-std, 08-examples, 10-cli 수준 기능이 안정화되어 있어야 한다.

산출물
- stage0 Rust 빌드: 기존 `gaut`/`cgen`을 정리한 기준 버전.
- stage1 C 백엔드: 전체 AST/타입을 C로 내리며 블록/함수 아레나 코드를 실제 삽입한다.
- stage1 바이너리: stage0이 생성한 C를 clang/zig로 빌드한 `gautc` (C생성+링크 CLI).
- 표준 라이브러리: `std/`를 C 백엔드와 일관되게 매핑하는 최소 러ntime shim(`runtime/c/` 등).
- 검증 스크립트: `scripts/self_host.sh`로 stage0→stage1→stage2 재빌드와 예제 실행을 자동화.

작업
1) C 백엔드 완성도 올리기
   - 모든 Expr/Stmt/Type 대응, 레코드/참조/if/블록 반환 처리, 내부 아레나 생성·reset 코드 삽입.
   - 출력 C 코드의 결정성 확보(정렬된 선언 순서, 안정된 map iteration).
   - 런타임 C 헬퍼 작성(아레나 alloc/reset, 문자열/바이트 조작)과 include 경로 구성.
2) CLI 확장(`gautc` 모드)
   - `gaut --emit c file.gaut -o out.c`와 `--build`(clang/zig 호출) 플래그 추가.
   - import 해석/표준 경로 처리 재사용, 에러 메시지 정비.
3) 표준 라이브러리/빌틴 정리
   - `std/str.gaut`/`std/bytes.gaut` 기능을 C 런타임 헬퍼와 매핑, `std/net.gaut`는 stub이더라도 타입 일치 유지.
   - 빌틴(print 등)과 런타임 shim 호출 규약 고정.
4) Self-host 루프 구축
   - stage0(Rust) → C 생성 → stage1 바이너리 빌드 → stage1로 다시 C 생성 → stage2 빌드.
   - stage1과 stage2가 동일 C 출력(또는 바이너리 동작 동일)인지 비교해 고정점 확인.

테스트/검증
- `cargo test`, `cargo clippy -- -D warnings` 기본 통과.
- `cargo test -p cgen`에 주요 언어 기능 커버리지 추가.
- `scripts/self_host.sh`에서 예제(`examples/*.gaut`)를 stage1/2 바이너리로 실행해 결과 비교.
- 결정성 체크: 동일 소스 → 동일 C 문자열 해시 비교.
