목표: 콘솔 예제와 통합 스크립트.

선행: 07-std.md 완료.

- 산출물
- examples/hello.gaut, examples/calc.gaut, examples/record.gaut
- scripts/run_examples.sh: 파서→타입체커→인터프리터→C 트랜스파일→clang 빌드·실행 파이프라인

테스트
- 스크립트 실행 결과가 예제 기대 출력과 일치
- 실패 시 적절한 오류 메시지 노출
