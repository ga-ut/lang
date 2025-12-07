목표: 타입/수명 체커 1차 구현.

선행: 03-parser.md 완료.

산출물
- crates/frontend/src/typecheck.rs

규칙
- 타입 명시 필수(추론 없음)
- 기본은 move, `copy`만 복사 허용
- use-after-move 에러
- 블록 수명 밖 참조/값 반환 금지 (global 예외)
- Effect는 무시

테스트
- 성공 케이스 3개(함수, 레코드, 참조)
- 실패 케이스 3개(use-after-move, 블록 탈출 참조, copy 누락 등)
