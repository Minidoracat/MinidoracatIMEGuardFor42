[h1]⌨️ Minidoracat IME Guard for B42[/h1]
[h3]By Minidoracat[/h3]

[hr][/hr]

[b]⚠️ 2 つで 1 セットの MOD です。サブスクライブだけでは何も起きません。Windows 用の小さな連携ツールも必要です（下記参照）。[/b]

[b]🎮 シングル／マルチ両対応。[/b]シングル：移動キーが IME に奪われなくなり、検索や名前入力欄では IME に戻ります。[b]マルチ：サーバーの MOD リストに本 MOD を入れる必要があります[/b]（PZ はサーバーのリストにある MOD だけ読み込みます）。サーバー側では何もせず、セーブにも触れません。

[h2]✨ これは何[/h2]
プレイ中は PZ を英語キーボードに固定し、チャット・検索・名前入力などで文字を打ち始めた瞬間に元の IME へ戻します。Microsoft IME、Google 日本語入力、注音、拼音、ハングルなど Windows の IME なら何でも対応。

[h2]🧟 なぜ IME で死ぬのか[/h2]
Windows では日本語などの IME が変換モードにあると、すべてのキーがまず IME に渡され、PZ はそれを無視します——WASD、Shift ダッシュ、スペース、E が全部効かなくなります。しかも PZ の操作キーは Windows の IME 切替ホットキー（Shift+Alt、Ctrl+Space）そのものなので、プレイ中に勝手に切り替わります。開発元はゲーム側で対処しないと明言しています。この MOD はそのためのものです。

[h2]🧰 使い方[/h2]
[olist]
[*] この MOD をサブスクライブして有効化。
[*] [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42/releases/latest]GitHub Releases[/url] から [b]pz-ime-guard.exe[/b] をダウンロードして実行。ウィンドウはなく、タスクトレイにキーキャップのアイコンが出るだけです（通常は「^」の中）。右下の小さなランプが状態：緑＝英語配列、橙＝入力中（IME 復帰済み）、灰＝PZ 未検出または一時停止、赤＝英語（米国）キーボード未インストール。
[*] ゲームを起動。ワールドに入った時にツールが動いていなければ一度だけ通知します。
[/olist]
Windows の「アプリ ウィンドウごとに異なる入力方式を設定する」を有効にすると、切替が PZ だけに限定されます。

[h2]🌐 対応 IME と言語[/h2]
[list]
[*] [b]保護対象の IME：[/b]中国語（注音・倉頡・嘸蝦米・Microsoft／搜狗拼音・RIME…）、日本語（Microsoft IME・Google 日本語入力…）、韓国語——中国語・日本語はプレイヤーの実証あり、韓国語は Microsoft の IME 資料で同じ挙動を確認。Windows 内蔵のベトナム語 Telex／VNI、インド系 Phonetic IME も仕組みは同じですが、ゲーム内では未検証です。
[*] [b]関与しない：[/b]英語各種、ロシア語、ドイツ語、フランス語、タイ語などの通常のキーボード配列——もともとこの問題はありません。
[*] [b]UI 言語：[/b]ツールのツールチップ・メニュー・ゲーム内通知は繁体中国語・簡体中国語・日本語・韓国語、それ以外は英語。Steam ページは英・繁・簡・日・韓。
[/list]

[h2]🦀 なぜ Rust か、なぜオープンソースか[/h2]
[list]
[*] 約 360 KB の単一 exe。.NET や Java などのランタイム不要、ダウンロードして実行するだけ。
[*] Rust はメモリ安全な言語で、バッファオーバーフロー系の脆弱性がありません。常駐時の負荷は 1 コアの約 0.3%、メモリ 2 MB 未満。
[*] 使うのは Windows の公開 API のみ：ウィンドウ列挙、キーボード配列の読み取り、切替メッセージの送信。キーボードフック、キー入力の読み取り、ネットワーク通信、レジストリ変更、他ウィンドウへの干渉は一切なし。MOD 本体は純粋な Lua で、「入力中」フラグを Zomboid フォルダに書くだけです。
[*] ソースは [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42]GitHub[/url] で公開。誰でも監査・自前ビルドできます。exe は毎回 GitHub Actions のクリーンな環境でビルドし、自動で VirusTotal に送信。SHA-256 とスキャン結果へのリンクは Release ページにあります。exe はコード署名していないため SmartScreen が「不明な発行元」と警告することがあります。不安なら cargo build してください。
[/list]

[h2]📋 MOD 情報[/h2]
[list]
[*] [b]Mod ID:[/b] MinidoracatIMEGuardFor42
[*] [b]対応バージョン:[/b] Build 42.20.4+
[*] [b]シングル / マルチ:[/b] 両対応（マルチはサーバーの MOD リストに要登録、上記参照）
[*] [b]プラットフォーム:[/b] ツールは Windows 専用。macOS／Linux にはこの問題はありません
[/list]

[h2]💬 フィードバック[/h2]
[list]
[*] [url=https://discord.gg/Gur2V67]Discord[/url]
[/list]

[h2]☕ 作者を支援[/h2]
この MOD は今後もずっと無料、ソースは GitHub で公開しています。気に入ったらコーヒーを一杯おごってください。支援はサーバーと MOD 開発に使います。
[url=https://ko-fi.com/minidoracat][img]https://raw.githubusercontent.com/Minidoracat/workshop-resources/refs/heads/main/badges/badge_kofi.png[/img][/url] [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42][img]https://raw.githubusercontent.com/Minidoracat/workshop-resources/refs/heads/main/badges/badge_github.png[/img][/url]

[b]#Minidoracat[/b]

Workshop ID: 3802890539
Mod ID: MinidoracatIMEGuardFor42
