# 1차 언어 스펙 초안 (Region/Context/Effect 제거 버전)

본 언어는 "블록 수명 기반 + 수식형"을 핵심으로 한다. Region/Context/Effect 같은 고급 개념은 2차 이후로 연기한다.

## 스코프/수명 규칙
- 블록 `{}` 하나가 유일한 수명 단위다. 블록 안에서 생성된 값/버퍼/참조는 블록 종료 시 모두 소멸한다.
- 전역 수명은 `global` 키워드로만 명시한다.
- 생성 블록 밖으로 값을 이동/반환하려면 전역이어야 한다. 블록 지역 값/참조를 바깥으로 돌려보내면 타입체커가 에러를 낸다.

## 문법 스케치 (BNF-ish)
```
Program      ::= Decl*
Decl         ::= ImportDecl | GlobalDecl | FuncDecl | TypeDecl | LetDecl
ImportDecl   ::= 'import' Ident
GlobalDecl   ::= 'global' Binding
LetDecl      ::= Binding
Binding      ::= ['mut'] Ident ':' Type '=' Expr
TypeDecl     ::= 'type' Ident '=' Type
FuncDecl     ::= Ident '(' Params? ')' ('->' Type)? '=' (Expr | Block)
Params       ::= Param (',' Param)*
Param        ::= ['mut'] Ident ':' Type
Block        ::= '{' Stmt* Expr? '}'
Stmt         ::= Binding | Assign | Expr
Assign       ::= Path '=' Expr
Path         ::= Ident ('.' Ident)*
Expr         ::= Literal
              | Path
              | 'copy' Expr
              | '&' Expr
              | FuncCall
              | IfExpr
              | Block
              | RecordLit
              | UnaryExpr
              | BinaryExpr
FuncCall     ::= Path '(' Args? ')'
Args         ::= Expr (',' Expr)*
IfExpr       ::= 'if' Expr 'then' Expr 'else' Expr
RecordLit    ::= '{' FieldInit (',' FieldInit)* '}'
FieldInit    ::= Ident ':' Expr
UnaryExpr    ::= ('-' | '!') Expr
BinaryExpr   ::= Expr Op Expr   // 우선순위: (), unary, *, /, +, -, <, ==, &&, ||
```
- 세미콜론은 없다.
- 블록은 `{}`로만 표현한다(들여쓰기 기반 문법은 후속 설탕 후보).
- 주석은 `// ...` 한 줄 주석만 제공한다.

## 타입 시스템 최소 코어
- 기본 타입: `i32`, `i64`, `u8`, `bool`, `Str`, `Bytes`, `Unit`(`()`)
- 레코드 타입: `type User = { name: Str, age: i32 }`
- 참조 타입: `&T` 하나만 제공. 참조는 생성 블록을 넘겨서 반환할 수 없다.
- 타입 추론은 없다. 모든 바인딩/매개변수/리턴에 타입을 명시한다.

## 바인딩과 값 이동 규칙
- 기본은 **move** semantics다. 바인딩을 다른 변수에 대입하면 원본은 더 이상 사용할 수 없다.
- 복사는 `copy expr`로만 허용한다(심플 규칙: 모든 타입이 기본 move, 필요 시 copy 명시).
- 가변 바인딩은 `mut`로 선언한다. 가변/불변 여부는 바인딩 수준에서만 구분한다(필드 단위 가변성은 없다).
- 참조 `&expr`는 해당 expr의 수명(블록) 안에서만 유효하다. 블록 밖으로 반환/저장 시 타입체커 오류.
- 대입 대상은 단순 식별자나 필드 경로(`a`, `a.b`)만 허용한다.

## 함수
- 형태: `name(params) -> Ret = expr` 또는 `= { ... }` 블록.
- 리턴 타입을 생략하면 기본 `Unit`(`()`)이다.
- 파라미터는 기본 불변이다. 파라미터를 직접 수정하려면 `mut` 파라미터로 선언하고, 그래도 여전히 블록 수명을 갖는다.
- 마지막 식이 리턴값이다(명시적 `return`은 없다).

## 모듈/임포트
- 한 파일이 한 모듈이다. 파일명 `foo.gaut` → 모듈 이름 `foo`.
- `import foo`는 같은 디렉터리 또는 표준 라이브러리 경로에서 `foo.gaut`을 불러온다.
- 네임스페이스 접근은 `foo.func`, `foo.Type` 형태.
- 접근제어/패키지/버전 개념은 없다(후속 과제).

## 전역
- `global name: Type = expr`로 선언한다.
- 전역은 프로그램 생존 범위로 유지된다. 전역을 참조하는 로컬 값/참조는 허용된다.

## 기본 표현식/연산자
- 리터럴: 정수(`123`), 불리언(`true`/`false`), 문자열(`"text"`), 바이트(`b"..."` TBD), Unit(`()`).
- 산술: `* / + -`, 비교 `< ==`, 논리 `&& ||`, 단항 `- !`.
- 조건식: `if cond then a else b` (표현식).
- 레코드: `{ x: 1, y: 2 }`, 필드 접근 `p.x`.
- 함수 호출: `f(a, b)`.
- 참조: `&value`, 역참조는 동일한 표기 없이 값처럼 사용(참조는 자동 역참조하지 않음; 참조 대상 타입이 그대로 노출됨).
- 복사: `copy expr`.

## 예제

### 1) Hello world (전역 + 블록 수명)
```
global greeting: Str = "hello"

print(msg: Str) = {
  // 실제 IO는 런타임에 의해 제공되는 함수라고 가정
}

main() = {
  msg: Str = greeting + " world"
  print(msg)
}
```

### 2) 간단 계산
```
add(a: i32, b: i32) -> i32 = a + b

main() = {
  x: i32 = 10
  y: i32 = 20
  sum: i32 = add(x, y)
  copy sum  // 값은 sum에 남고, 이후에도 sum 사용 가능
}
```

### 3) 구조체 + 참조 + 이동/복사
```
type Point = { x: i32, y: i32 }

shift(p: Point, dx: i32, dy: i32) -> Point = {
  mut moved: Point = p       // move: p는 이후 사용 불가
  moved.x = moved.x + dx
  moved.y = moved.y + dy
  moved                      // 블록 마지막 식 반환
}

length_x(p: &Point) -> i32 = p.x

main() = {
  origin: Point = { x: 0, y: 0 }
  p1: Point = shift(origin, 5, 0)
  px: i32 = length_x(&p1)        // 참조는 블록 내에서만 유효
  copy px
}
```

## 용어 정리
- 블록 수명: `{}`로 감싼 영역. 생성된 값/참조는 블록 종료 시 소멸.
- 전역: `global`로 선언된 값. 프로그램 전체 수명.
- 참조: `&T`. 소유권을 이동하지 않고 읽기 접근만 공유한다.
- 이동(move): 값을 다른 바인딩으로 넘기면 원본을 더 이상 사용할 수 없는 규칙.
- 복사(copy): `copy expr`로 명시적으로 새 값을 만든다.
