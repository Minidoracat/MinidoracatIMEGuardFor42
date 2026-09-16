[h1]⌨️ Minidoracat IME Guard for B42[/h1]
[h3]By Minidoracat[/h3]

[hr][/hr]

[b]⚠️ 두 부분으로 구성된 모드입니다. 구독만으로는 아무 일도 일어나지 않으며, 작은 Windows 보조 도구도 필요합니다(아래 참고).[/b]

[h2]✨ 이 모드는[/h2]
플레이 중에는 PZ를 영어 키보드 배열로 고정하고, 채팅·검색·이름 입력 등 글자를 치기 시작하는 순간 원래 쓰던 IME로 되돌립니다. Microsoft 한글 IME, 중국어(주음·병음), 일본어 등 Windows IME라면 무엇이든 동작합니다.

[h2]🧟 왜 IME 때문에 죽는가[/h2]
Windows에서 한국어·중국어·일본어 IME가 조합 모드에 있으면 모든 키 입력이 먼저 IME로 넘어가고, PZ는 그 키를 그냥 버립니다. WASD, Shift 달리기, 스페이스, E 전부 먹통이 됩니다. 게다가 PZ의 조작 키는 Windows의 입력기 전환 단축키(Shift+Alt, Ctrl+Space)와 겹쳐서 게임 도중 저절로 바뀌기도 합니다. 개발사는 게임 쪽에서 처리하지 않겠다고 밝혔습니다. 이 모드는 그 문제를 위한 것입니다.

[h2]🧰 사용 방법[/h2]
[olist]
[*] 이 모드를 구독하고 활성화합니다.
[*] [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42/releases/latest]GitHub Releases[/url]에서 [b]pz-ime-guard.exe[/b]를 내려받아 실행합니다. 창은 없고 시스템 트레이에 키캡 아이콘만 표시됩니다(보통 ^ 안에 숨어 있음). 모서리의 작은 램프: 초록＝영어 배열, 주황＝입력 중(IME 복원됨), 회색＝PZ 미검출 또는 일시 정지, 빨강＝영어(미국) 키보드 미설치.
[*] 게임을 시작합니다. 월드에 들어갈 때 도구가 실행 중이 아니면 한 번 알려 줍니다.
[/olist]
Windows의 「앱 창마다 다른 입력기 설정」을 켜 두면 전환이 PZ에만 적용됩니다.

[h2]🌐 지원 IME 및 언어[/h2]
[list]
[*] [b]보호하는 IME:[/b] 한국어(Microsoft 한글 IME 등), 중국어(주음·창힐·병음·RIME…), 일본어(Microsoft IME·Google 일본어 입력…) — 중국어·일본어는 플레이어가 실증했고, 한국어는 Microsoft IME 문서로 같은 동작을 확인했습니다. Windows 내장 베트남어 Telex／VNI, 인도계 Phonetic IME도 같은 원리이지만 게임 내 검증은 아직입니다.
[*] [b]관여하지 않음:[/b] 영어 각 변형, 러시아어, 독일어, 프랑스어, 태국어 등 일반 키보드 배열 — 원래 이 문제가 없습니다.
[*] [b]UI 언어:[/b] 도구 툴팁·메뉴·게임 내 안내는 한국어, 번체·간체 중국어, 일본어, 그 외는 영어. Steam 페이지는 영·번·간·일·한.
[/list]

[h2]🦀 왜 Rust인가, 왜 오픈 소스인가[/h2]
[list]
[*] 약 450 KB 단일 exe. .NET, Java 등 런타임 설치 불필요, 내려받아 바로 실행.
[*] Rust는 메모리 안전 언어로 버퍼 오버플로 계열 취약점이 없습니다. 상주 시 CPU 약 0.3%(코어 1개 기준), 메모리 2 MB 미만.
[*] Windows 공개 API만 사용: 창 열거, 키보드 배열 읽기, 전환 메시지 1개 전송. 키보드 후킹, 키 입력 읽기, 네트워크, 레지스트리 변경, 다른 창 간섭 없음. 모드 본체는 순수 Lua이며 「입력 중」 플래그를 Zomboid 폴더에 쓰기만 합니다.
[*] 소스는 [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42]GitHub[/url]에 공개. 누구나 검토하거나 직접 빌드할 수 있습니다. exe는 코드 서명이 없어 SmartScreen이 「알 수 없는 게시자」 경고를 띄울 수 있습니다. 불안하면 직접 cargo build 하세요.
[/list]

[h2]📋 모드 정보[/h2]
[list]
[*] [b]Mod ID:[/b] MinidoracatIMEGuardFor42
[*] [b]지원 버전:[/b] Build 42.20.4+
[*] [b]싱글 / 멀티:[/b] 모두 지원(클라이언트 전용, 서버에는 필요 없음)
[*] [b]플랫폼:[/b] 도구는 Windows 전용. macOS／Linux에는 이 문제가 없습니다
[/list]

[h2]💬 피드백[/h2]
[list]
[*] [url=https://discord.gg/Gur2V67]Discord[/url]
[/list]


[b]#Minidoracat[/b]
