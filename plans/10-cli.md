목표: `.gaut` 파일을 직접 실행하는 CLI(`gaut`) 추가. `gaut examples/hello.gaut` 실행 시 콘솔에 결과/print가 나타나야 한다.

산출물
- `crates/cli` 바이너리 크레이트 (workspace member)
- 실행: `cargo run -p cli -- examples/hello.gaut`
- 기능: 파일 로딩 → import 해석(동일 디렉터리 + std 경로) → 파싱 → 타입체크 → 인터프리트 → stdout 출력
- 내장 `print`/`println` 빌틴을 인터프리터에 추가(이름 기반 매핑)하여 실제 콘솔 출력 가능하게 하기

작업
1) 워크스페이스에 `crates/cli` 추가, 의존성: frontend, interp, runtime, anyhow/ariadne(선택)로 에러 표시.
2) 파일 로더 구현:
   - 기본 확장자 `.gaut`
   - import 해석: 같은 디렉터리 우선, 없으면 `std/` 경로(`$REPO/std`)에서 찾기
   - 순환 import 방지, 중복 로드 캐싱
3) 타입체크/인터프리터 연결:
   - 로드한 Program을 그대로 타입체커에 통과시킨 후 인터프리터 실행
   - 실패 시 친숙한 에러 메시지와 non-zero 종료코드
4) 빌틴 IO 추가:
   - 인터프리터에 `print`/`println` 이름을 감지해 stdout에 문자열(Value::Str) 출력 후 Unit 반환하도록 특별 처리
   - 예제 `hello.gaut`에서 실제 출력 발생 확인
5) UX
   - `gaut <file.gaut>` 실행 시 결과 값을 `Debug/Display`로 출력
   - `--ast`, `--c` 등 디버그 옵션은 후속 (선택)

테스트
- `cargo run -p cli -- examples/hello.gaut` 출력 확인(“hello world”) 
- `cargo run -p cli -- examples/calc.gaut` 출력이 `30`인지 확인
- `cargo test -p cli`에 최소 smoke 테스트 추가(파일 로드 → 실행 → 결과 검증)
