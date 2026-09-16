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

[h2]🔒 하는 것과 하지 않는 것[/h2]
[list]
[*] 모드 본체는 순수 Lua이며 「입력 중」 플래그를 Zomboid 폴더에 쓰기만 합니다. 네이티브 코드는 없습니다.
[*] 도구는 PZ가 전면 창일 때만 Windows 공식 메시지로 그 창의 배열을 바꿉니다. 키보드 후킹, 키 전송, 레지스트리 변경, 다른 창 간섭은 전혀 없습니다.
[*] 오픈 소스: [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42]GitHub[/url]
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
