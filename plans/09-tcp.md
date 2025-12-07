목표: TCP 예제(선택)로 IO 러퍼 검증.

선행: 08-examples.md 완료, Net thin wrapper 준비.

- 산출물
- runtime/net.rs: net_listen/accept/read/write thin wrapper
- examples/tcp_echo.gaut
- (향후) 수동/간단 자동 테스트 스크립트

작업
- 러퍼 API를 스펙에 맞춰 노출
- echo 핸들러 예제 작성(블록 스코프로 수명 제한)

테스트
- 로컬에서 echo 요청→응답 확인(수동 또는 간단 스크립트)
