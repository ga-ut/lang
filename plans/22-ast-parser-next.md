AST 테이블/파서 후속 작업 실행 계획 (3단계 분해)

1) AST 테이블 레이아웃 정의
   - 노드별 바이트 포맷/stride 확정(Decl/Expr/Type 등)
   - 리스트 노드 포맷(Head/Tail) 정의
   - 각 테이블의 기본 empty/append 규약 확정

2) AST 테이블 유틸 추가
   - 공통 encode/decode(i32) 헬퍼 정리
   - 테이블별 append/get 함수 구현
   - smoke: 간단 노드 1개 기록/읽기 검증

3) 파서 구현(타입/표현식/선언)
   - 타입 파싱부터 시작해 Expr/Decl 순으로 확장
   - AST 테이블에 노드 적재 및 인덱스 반환
   - 최소 입력 스모크 파싱 확인

4) 기존 테스트 외 스모크 실행 경로 추가
   - Gaut 쪽에 smoke_* 호출하는 임시 entry 추가
   - 확인 후 제거 또는 유지 여부 결정
