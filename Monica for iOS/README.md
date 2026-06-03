# Monica for iOS

Monica for iOS 是 Monica 的 SwiftUI 客户端工作区。

当前开发路径已经锁定：

- MDBX 优先，不使用临时 SQLite 主存储。
- iOS 17+。
- iPhone 优先。
- Rust/Swift 桥接使用 UniFFI。
- 首阶段只验证 Tiga `Multi Type`。
- 首个 TestFlight 包含 MDBX、本地 Vault、WebDAV 和较完整 AutoFill。
- Passkey、KeePass、Bitwarden、附件、Plus 资源解锁后置；Plus 不进入 IAP。

## 当前目录结构

```text
Monica for iOS/
  README.md
  Monica.xcodeproj/
  cross-platform-migration-plan.md
  findings.md
  progress.md
  task_plan.md
  App/
    MonicaApp/
  Extensions/
    MonicaAutoFillExtension/
  Tests/
    MonicaTests/
  Scripts/
    build-mdbx-xcframework.sh
    generate-mdbx-swift-bindings.sh
  Artifacts/
    MDBX/
  Generated/
    MDBXUniFFI/
  SwiftPackages/
    MonicaCore/
    MonicaMDBX/
    MonicaSecurity/
    MonicaStorage/
    MonicaSync/
    MonicaUI/
```

## 本地化和图标

- App、AutoFill Extension、权限说明和用户可见错误文案已切换为中文；技术名仍保留英文，例如 Monica、MDBX、TOTP、WebDAV、Keychain、App Group、UniFFI。
- iOS AppIcon 复用 Android launcher 图标，源文件为 `Monica for Android/app/src/main/res/drawable-nodpi/monica_launcher.png`。
- 生成后的 iOS 图标位于 `App/MonicaApp/Assets.xcassets/AppIcon.appiconset/`，包含 iPhone 所需的 20/29/40/60 pt 尺寸和 1024 px marketing 图标。
- 图标 PNG 已重新生成成不透明资源，并通过 Xcode asset catalog 接入 `Monica` target 的 Resources build phase。

## MDBX UniFFI 技术验证

Rust 侧桥接 crate 位于：

```text
mdbx/crates/mdbx-ios-ffi
```

生成 Swift binding：

```bash
cd "Monica for iOS"
Scripts/generate-mdbx-swift-bindings.sh
```

脚本会：

1. 构建 `mdbx-ios-ffi` 的动态库。
2. 调用 `uniffi-bindgen-swift` 生成 Swift binding、header 和 modulemap。
3. 将生成结果写到 `Generated/MDBXUniFFI`。

`build-mdbx-xcframework.sh` 会把 iOS 可用的 Swift binding 同步到 `SwiftPackages/MonicaMDBX/Sources/MonicaMDBX/Generated`，并用条件编译避免 macOS SwiftPM 测试误编译 iOS-only FFI。

如果缺少 UniFFI CLI：

```bash
cargo install uniffi --version 0.31.1 --locked --features cli
```

生成 iOS XCFramework：

```bash
cd "Monica for iOS"
Scripts/build-mdbx-xcframework.sh
```

脚本会：

1. 确认 Xcode、iOS SDK、Cargo/Rust 和 iOS Rust 标准库可用。
2. 生成 UniFFI Swift/header/modulemap。
3. 分别构建 `aarch64-apple-ios`、`aarch64-apple-ios-sim` 和 `x86_64-apple-ios` 的 Rust staticlib。
4. 输出 `Artifacts/MDBX/MonicaMDBXGenerated.xcframework`。
5. 将生成的 Swift binding 同步到 `MonicaMDBX` package。

当前开发机已使用 rustup toolchain，并安装了 `aarch64-apple-ios`、`aarch64-apple-ios-sim` 和 `x86_64-apple-ios` targets。

注意：不要把 `Scripts/build-mdbx-xcframework.sh` 和 Xcode build 并行运行。脚本会替换 `MonicaMDBXGenerated.xcframework`，Xcode 在替换期间读取产物会失败；先重建 XCFramework，再运行 Xcode build。

## 验证命令

Rust smoke：

```bash
cd ../mdbx
cargo test -p mdbx-ios-ffi
```

Swift package 结构检查：

```bash
cd "Monica for iOS/SwiftPackages/MonicaCore"
swift test

cd "../MonicaMDBX"
swift test

cd "../MonicaStorage"
swift test

cd "../MonicaSecurity"
swift test

cd "../MonicaSync"
swift test

cd "../MonicaUI"
swift test
```

iOS simulator build：

```bash
xcrun simctl list devices available

xcodebuild \
  -project "Monica for iOS/Monica.xcodeproj" \
  -scheme Monica \
  -configuration Debug \
  -sdk iphonesimulator \
  -destination "id=<iPhone simulator UUID>" \
  CODE_SIGNING_ALLOWED=NO \
  build
```

MDBX iOS simulator XCTest：

```bash
xcrun simctl list devices available

xcodebuild test \
  -project "Monica for iOS/Monica.xcodeproj" \
  -scheme Monica \
  -configuration Debug \
  -sdk iphonesimulator \
  -destination "id=<iPhone simulator UUID>" \
  CODE_SIGNING_ALLOWED=NO
```

`xcodebuild test` 必须使用具体 simulator，不能使用 `generic/platform=iOS Simulator`。当前已验证的模拟器是 `iPhone 17 Pro`，iOS 26.5，UDID `4F179679-A513-4C20-A935-6164CBCE2711`。

App 内技术验证：

1. 在 Xcode 中运行 `Monica` scheme。
2. 打开「保险库」页。
3. 点击「运行 MDBX 检查」。
4. 预期结果是创建临时 `.mdbx` vault、写入 project-scoped login entry、重开 vault 并读回同一条 entry。

本地 Vault 会话验证：

1. 在「保险库」页输入保险库名称和主密码。
2. 点击「创建 MDBX 保险库」。
3. 预期状态切换为「已解锁」，活动保险库名称显示为输入名称。
4. 可点击「打开已有保险库」选择 `.mdbx` 文件打开。
5. 解锁后可在「密码」区块创建登录条目并在列表中看到它。
6. 可用「搜索」按标题、用户名或 URL 过滤当前会话登录条目。
7. 可点击「生成密码」为新增登录草稿生成默认 20 位随机密码；生成只填入表单，不会自动保存条目。
8. 点击列表里的登录条目后，可在「已选登录项」区块编辑标题、用户名、密码和 URL，也可点击「生成密码」为编辑草稿换一个新密码；只有点击保存后才写入 MDBX。
9. 可在「已选登录项」区块切换「收藏」，收藏状态会写入登录条目的加密 payload；普通编辑会保留收藏状态，收藏不会进入 AutoFill 加密索引、secret snapshot 或 credential identities。密码、安全笔记、TOTP、银行卡和证件元数据列表默认收藏优先，并可用「只看收藏」只显示收藏项，筛选可与搜索叠加。
10. 可用「删除登录项」将条目软删除到「最近删除」，再用「恢复」恢复。
11. 解锁后可在「安全笔记」区块创建安全笔记；笔记支持按标题或正文搜索，点击列表项进入「已选笔记」编辑，也可软删除到「已删除笔记」并恢复。
12. 解锁后可在「TOTP」区块创建 TOTP 条目；当前支持粘贴 `otpauth://` URI 或通过相机扫描二维码导入标题、密钥、发行方、账号、周期、位数、算法到草稿表单，也支持手动存储和编辑，支持搜索、软删除到「已删除 TOTP」并恢复，并可根据存储 seed 每秒刷新当前验证码和剩余秒数。
13. 解锁后可在「银行卡」区块创建银行卡条目；当前支持持卡人、卡号、有效期、CVV、发卡行、卡组织和备注，支持搜索、选择编辑、软删除到「已删除银行卡」并恢复。列表摘要只显示卡组织、发卡行和末四位。
14. 解锁后可在「证件」区块创建证件元数据条目；当前支持证件类型、姓名、证件号、签发方、国家、签发日期、到期日期和备注，支持搜索、选择编辑、软删除到「已删除证件」并恢复。证件号和备注留在 MDBX payload，不进入 AutoFill。
15. 解锁后可点击「锁定保险库」清空当前活动会话、搜索状态、编辑状态和回收站视图状态。
16. App 进入后台会立即锁定 vault；进入 inactive 状态时会显示隐私遮罩，避免系统截图暴露敏感内容。
17. 解锁后默认 5 分钟无活动会自动锁定；用户点击和条目操作会刷新自动锁定窗口。「设置」里可以选择 1 分钟、5 分钟、15 分钟或 30 分钟。
18. 创建或打开 vault 失败时会保持锁定、清空主密码输入，并保留失败状态用于提示。
19. 如果 App 配置了 AutoFill index/secret store、identity store 和 key provider，创建、编辑、删除或恢复登录条目后会自动刷新加密 AutoFill 索引、加密 secret snapshot 和系统 credential identities；索引文件不包含明文域名、标题或账号，secret snapshot 不包含明文账号或密码。安全笔记、TOTP、银行卡、证件元数据和所有条目的收藏状态不会进入 AutoFill 索引。
20. 「设置」会显示 Vault Keychain 状态，并提供「启用 Keychain 解锁」/「使用 Keychain 解锁」操作。App 层会在手动解锁后生成本地 security key material、注册到当前 MDBX vault，并通过 `AppVaultKeychainService` 保存到 Keychain；认证后读取该 material，再通过 MDBX `security_key` 打开 remembered vault。这个流程不保存主密码，也不通过 resolver 还原主密码。
21. 「设置」的 WebDAV 区块可配置服务器 URL、用户名、密码和远端文件；解锁 vault 后可手动上传当前 active `.mdbx` 文件，或下载远端备份生成恢复预览。上传时会同时写入同名 `.sha256` sidecar，下载时优先使用 `X-Monica-Backup-SHA256` header，header 缺失时回退读取 `.sha256` sidecar 做完整性校验。确认恢复需要输入恢复 vault 密码，App 会先用临时候选 `.mdbx` 文件验证备份可打开，验证通过后才释放并锁定当前 vault session，再用临时文件替换本地 `.mdbx` 文件；验证失败时不会覆盖本地 vault。

## App / Extension 骨架

当前已创建源码骨架：

- `App/MonicaApp`：SwiftUI App 入口、iPhone-first tab shell、本地 vault create/open/lock 会话雏形、密码条目 create/list/search/update/delete/restore/favorite、登录密码生成填表、安全笔记 create/list/search/update/delete/restore/favorite、TOTP create/list/search/update/delete/restore/favorite、银行卡 create/list/search/update/delete/restore/favorite、证件元数据 create/list/search/update/delete/restore/favorite、多类型列表收藏优先排序和“只看收藏”筛选、`otpauth://` URI 导入填表、AVFoundation 相机二维码扫描入口、扫描错误可读提示和当前验证码每秒刷新、后台锁定、inactive 隐私遮罩、可配置自动锁定策略、vault Keychain/LocalAuthentication + MDBX `security_key` 解锁边界、主 App 加密 AutoFill 索引和 secret snapshot 生成、条目变更自动同步、AutoFill credential identities 同步、生产路径 App Group index/secret store / Keychain-backed index key provider / `ASCredentialIdentityStore` wiring、MDBX round-trip 检查入口、App Group entitlement、中文用户可见文案和 Android launcher 图标复用。TOTP 相机权限/画面仍需签名真机验证。
- `Extensions/MonicaAutoFillExtension`：Credential Provider Extension 主类、Info.plist、App Group entitlement；可从 App Group 读取加密 AutoFill 索引和 secret snapshot，经 LocalAuthentication/Keychain 解锁 key 后做域名匹配和列表搜索，并在用户选择记录后通过 `ASPasswordCredential` 交给系统填充。
- `Tests/MonicaTests`：Xcode XCTest target，自动验证 MDBX login/note/totp/card/identity UniFFI round-trip、MDBX local security key setup/open round-trip、Storage entry repository 真实 MDBX round-trip、App vault session create/open/lock、vault 失败路径安全清理、首条密码创建/list、登录条目搜索、登录条目编辑、删除/恢复、登录条目收藏且不改 payload、安全笔记/TOTP/银行卡/证件元数据收藏且不改 payload、多类型列表收藏优先排序和 Favorites Only 筛选、登录密码生成只填草稿不落库、安全笔记创建/搜索/编辑/删除/恢复、TOTP 创建/搜索/编辑/删除/恢复、银行卡创建/编辑/删除/恢复、证件元数据创建/编辑/删除/恢复、TOTP seed 生成验证码、TOTP URI 导入填表且不落库、扫描到的 TOTP QR payload 导入填表且不落库、无效 TOTP QR 使用可读提示且不改草稿、TOTP 剩余秒数计算、后台自动锁定、inactive 隐私遮罩、自动锁定窗口、自动锁定策略切换、认证 AutoFill index key 解密 Storage 索引 payload、AutoFill index key material provider 创建/复用、App 保存 Keychain-protected security key material 且不持久化主密码、Keychain unlock 认证后用 MDBX `security_key` 打开 remembered vault、主 App 加密 AutoFill 索引生成、条目变更自动同步索引、主 App 写入可供 Extension 解锁填充的 AutoFill secret snapshot、条目变更同步 AutoFill credential identities，以及 WebDAV active vault 上传、恢复预览、确认恢复前打开验证和失败不覆盖本地 vault。
- `SwiftPackages/MonicaCore`：iOS 17+ / MDBX-first 基线信息、RFC 6238 TOTP 基础生成器、`otpauth://` URI parser 和安全随机密码生成器，覆盖 SHA1/SHA256/SHA512、Base32 secret 规范化、period、digits、algorithm、URI 错误校验、密码字符集约束和无效生成策略。
- `SwiftPackages/MonicaStorage`：本地 vault repository、MDBX security key setup/open 边界、密码/安全笔记/TOTP/银行卡/证件元数据条目 repository 边界、多类型条目收藏边界、AutoFill AES-GCM 加密索引 codec、AutoFill 加密 secret snapshot codec、解密索引搜索/域名匹配和 App Group 文件存储边界、vault close/release 边界，App vault 管理通过这里调用 `MonicaMDBX`。
- `SwiftPackages/MonicaSync`：WebDAV 首版基础，包含可注入 transport、URLSession 生产 transport、Basic auth、PUT 上传、GET 下载、SHA-256 完整性 header、`.mdbx.sha256` sidecar fallback、下载完整性校验和恢复预览；App 层已通过 Settings 接入手动备份、恢复预览、恢复前打开验证和确认恢复。
- `SwiftPackages/MonicaSecurity`：vault wrapped key 与 AutoFill index key 的 Keychain/LocalAuthentication 边界，AutoFill index key 当前强制 32 字节并需要认证后读取。

这些文件已经纳入 `Monica.xcodeproj`。真机第一轮已确认 `Evangelion` 设备可被 Xcode/CoreDevice 识别，`CODE_SIGNING_ALLOWED=NO` 的 `iphoneos` arm64 build 通过，App、debug dylib 和 AutoFill Extension 产物均为 arm64 Mach-O。当前尚不能安装运行，因为 Xcode Accounts 没有 Team `B6R6XP99R2` 的有效账号，也没有 `takagi.ru.monica` / `takagi.ru.monica.autofill` development provisioning profiles；未签名产物安装会被设备以 `0xe800801c (No code signature found.)` 拒绝。下一步是在真机/签名环境补齐 development team、App Group、Keychain access group 和 Credential Provider profiles 后，确认 App Group、Keychain access group、Credential Provider、QuickType identity 展示、TOTP 相机权限/扫描画面，以及 Keychain/LocalAuthentication + MDBX `security_key` 解锁可运行；同时继续打磨 TOTP 扫描视觉细节、本地 Vault 多类型条目体验，并做 WebDAV 真实服务兼容测试。无签名模拟器测试使用 `CODE_SIGNING_ALLOWED=NO` 时 App Group container 不可用是预期现象，生产工厂会在这种情况下不启用 AutoFill index/secret store/provider。
