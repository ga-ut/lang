부족한 부분(현재 상태 기준)

1) Gaut 컴파일러 핵심 모듈이 스텁 상태
   - compiler/ast.gaut: Rust frontend::ast 대응 모델 미이식
   - compiler/parser.gaut: 토크나이저/파서 미구현
   - compiler/typecheck.gaut: 타입/무브/블록 수명 규칙 미구현
   - compiler/cgen.gaut: 실제 C 생성 로직 미구현

2) 컴파일 파이프라인 미연결
   - compiler/main.gaut가 read_file/parse/typecheck/cgen을 호출하지 않고 emit_stub()만 사용
   - --build 플래그는 파싱만 하고 동작 규약 없음

3) 문자열/바이트 프리미티브 부족
   - parser 구현에 필요한 bytes_len/bytes_push/bytes_slice 미제공
   - str_len/str_byte_at/str_slice는 있으나 bytes 조작이 제한적

4) self-host strict 경로 불가
   - stage0/1/2 해시 비교는 준비되어 있으나 컴파일러가 유효 C를 내지 못해 실패

5) 테스트/결정성 검증 부족
   - Gaut 컴파일러 쪽 스모크/골든 테스트 부재
   - C 출력 결정성(정렬/안정적 출력) 보장 미검증
