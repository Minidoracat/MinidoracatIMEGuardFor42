[h1]⌨️ Minidoracat IME Guard for B42[/h1]
[h3]By Minidoracat[/h3]

[hr][/hr]

[b]⚠️ 2 つで 1 セットの MOD です。サブスクライブだけでは何も起きません。Windows 用の小さな連携ツールも必要です（下記参照）。[/b]

[h2]✨ これは何[/h2]
プレイ中は PZ を英語キーボードに固定し、チャット・検索・名前入力などで文字を打ち始めた瞬間に元の IME へ戻します。Microsoft IME、Google 日本語入力、注音、拼音、ハングルなど Windows の IME なら何でも対応。

[h2]🧟 なぜ IME で死ぬのか[/h2]
Windows では日本語などの IME が変換モードにあると、すべてのキーがまず IME に渡され、PZ はそれを無視します——WASD、Shift ダッシュ、スペース、E が全部効かなくなります。しかも PZ の操作キーは Windows の IME 切替ホットキー（Shift+Alt、Ctrl+Space）そのものなので、プレイ中に勝手に切り替わります。開発元はゲーム側で対処しないと明言しています。この MOD はそのためのものです。

[h2]🧰 使い方[/h2]
[olist]
[*] この MOD をサブスクライブして有効化。
[*] [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42/releases/latest]GitHub Releases[/url] から [b]pz-ime-guard.exe[/b] をダウンロードして実行。タスクトレイに小さな四角が出るだけです：緑＝英語配列、橙＝入力中（IME 復帰済み）、灰＝PZ 未検出または一時停止、赤＝英語（米国）キーボード未インストール。
[*] ゲームを起動。ワールドに入った時にツールが動いていなければ一度だけ通知します。
[/olist]
Windows の「アプリ ウィンドウごとに異なる入力方式を設定する」を有効にすると、切替が PZ だけに限定されます。

[h2]🔒 やること・やらないこと[/h2]
[list]
[*] MOD 本体は純粋な Lua で、「入力中」フラグを Zomboid フォルダに書くだけ。ネイティブコードは含みません。
[*] ツールは PZ が前面にある時だけ、Windows 公式のメッセージでそのウィンドウの配列を切り替えます。キーボードフック、キー送信、レジストリ変更、他ウィンドウへの干渉は一切なし。
[*] ソース公開：[url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42]GitHub[/url]
[/list]

[h2]📋 MOD 情報[/h2]
[list]
[*] [b]Mod ID:[/b] MinidoracatIMEGuardFor42
[*] [b]対応バージョン:[/b] Build 42.20.4+
[*] [b]シングル / マルチ:[/b] 両対応（クライアント側のみ、サーバーには不要）
[*] [b]プラットフォーム:[/b] ツールは Windows 専用。macOS／Linux にはこの問題はありません
[/list]

[h2]💬 フィードバック[/h2]
[list]
[*] [url=https://discord.gg/Gur2V67]Discord[/url]
[/list]


[b]#Minidoracat[/b]
