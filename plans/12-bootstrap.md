목표: Gaut로 Gaut 컴파일러를 작성해 stage1/2 self-host 루프를 검증하고, 런타임/표준 라이브러리와 cgen 커버리지를 확장한다.

선행: 11-self-host 완료(현재 Rust→C→clang 파이프라인, 예제 결정성 통과).

산출물
- Gaut로 작성된 최소 컴파일러 소스(파서/타입체커/cgen) + 빌트인/IO 브리지 정의.
- stage0(Rust)→C→clang 빌드된 gautc1, gautc1→gautc2 hash/동작 비교 스크립트.
- 확장된 C 런타임/stdlib(bytes/str)와 cgen 테스트 커버리지, 경고 정리.

작업 A: Gaut-작성 컴파일러와 self-host 루프
1) 서브셋 고정: Gaut 컴파일러가 필요로 하는 문법/타입 범위 문서화(함수/레코드/조건/블록/참조/이동/복사; 제네릭/매크로 없음). 허용된 호스트 빌트인(IO/FS) 명시.
2) 모듈 뼈대: `compiler/ast.gaut`, `parser.gaut`, `typecheck.gaut`, `cgen.gaut`, `main.gaut`에 역할만 스텁 함수로 배치.
3) FFI 빌트인: runtime C에 `builtin.read_file(path: Str)->Str`, `write_file(path: Str, data: Str)->Unit`, `args()->Bytes` 추가, Gaut 측 시그니처 연결.
4) stage 루프 스크립트: `scripts/self_host.sh` 확장 → (a) stage0 emits gautc1.c, (b) clang→gautc1, (c) gautc1 emits gautc2.c, (d) clang→gautc2, (e) hash/diff gautc1.c vs gautc2.c 및 예제 실행. `SELF_HOST_SKIP=1` 환경변수로 생략 옵션 제공.
5) 결정성: Gaut 컴파일러에서 decl/field 정렬, map 순회 고정; C 출력 canonicalize(트레일링 스페이스 제거 등) 후 sha 비교.

작업 B: 런타임/표준 라이브러리 확장 + cgen 커버리지
1) Str/Bytes: length/slice/concat/compare를 runtime C + std 바인딩에 추가, cgen에서 bytes concat 이미 매핑되었는지 확인 및 테스트.
2) Net 스텁: 타입 유지, 선택적 no-op echo helper로 예제 실행 가능하게 문서화.
3) cgen 기능: 필드 기록/참조 접근 `->` 검증, 중첩 레코드/블록/조건 케이스 테스트 추가, decl/필드 이름 정렬로 결정성 강화.
4) 경고 정리: parser Token 가시성 수정, `IndexMap::remove` → `shift_remove`, 필요 시 lint 설정 조정.
5) 테스트: `cargo test -p cgen -p cli`, runtime C 단위테스트 또는 Rust FFI 스모크 추가, self_host 스크립트에 확장된 예제/해시 체크 포함.

검증 기준
- `scripts/self_host.sh`가 stage1/2 루프와 해시 비교까지 수행(옵션 포함), 예제 실행 성공.
- Gaut-작성 컴파일러 소스가 저장소에 존재하고, stage0→C→clang 빌드 성공(부분 구현이라도 컴파일/링크 가능).
- Str/Bytes 확장 기능을 사용하는 테스트 통과 및 cgen 결정성 보장.
