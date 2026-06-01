# Monica for iOS 跨平台迁移计划

日期：2026-05-30

## 1. 目标

使用 SwiftUI 重建 Monica for iOS，同时保留项目最重要的产品身份：

- 本地优先的密码管理。
- 用户自主管理同步和备份。
- 强加密 vault。
- 与 KeePass 以及 Monica 新 MDBX 格式保持兼容。
- 支持 iOS 原生 AutoFill，并在后续支持 Passkey。

这不是一次 Kotlin 到 Swift 的逐文件翻译。Android 项目中包含大量平台专属能力，例如 AutofillService、IME、AccessibilityService、WorkManager 和 Android Credential Provider。iOS 版必须使用 iOS 原生机制替代，无法替代的部分需要明确降级或重新设计。

## 2. Brainstorming 结论

### 方案 A：完整功能对齐迁移

在首版发布前，把 Android 的所有功能都用 SwiftUI 和 iOS 系统能力重建。

优点：

- 对现有 Android 用户来说功能落差最小。
- 对外宣传时更容易强调全平台一致。

缺点：

- 交付风险极高。
- iOS 无法原样实现部分 Android 能力。
- 安全关键功能可能被范围压力拖慢。

结论：不适合作为第一版路线。

### 方案 B：iOS MVP + 严格格式兼容

先发布聚焦版 iOS：本地 vault、解锁、核心条目、TOTP、备份和基础 AutoFill。所有数据格式和测试都向 Android / MDBX 兼容靠拢。

优点：

- 最快得到可用且安全的 iOS 版本。
- UI 和交互可以遵循 iOS 习惯，而不是复制 Android。
- 能尽早验证 AutoFill、App Group 和 MDBX 的关键假设。

缺点：

- 部分 Android 功能需要延后。
- 需要严格控制 MVP 范围。

结论：推荐采用。

### 方案 C：MDBX 优先技术原型

先把 Rust MDBX workspace 编译成 iOS 可用库，再做一个很薄的 SwiftUI 测试 App。

优点：

- 最有利于长期格式兼容。
- 能尽早暴露 FFI、构建、内存和格式边界问题。

缺点：

- 用户可见功能进展较慢。
- AutoFill 和完整产品流程会延后。

结论：适合作为并行技术验证，不建议作为唯一主线。

## 3. 推荐策略

产品主线改为 **MDBX 优先**：先完成 Rust MDBX 到 iOS 的技术验证，再推进完整 SwiftUI App。

iOS 版应该是原生 SwiftUI 客户端，采用清晰模块化架构。存储设计必须直接围绕 MDBX，而不是先做临时 SQLite 主存储。首阶段目标是证明 iOS 真机和模拟器可以通过 Swift/UniFFI 调用 Rust MDBX，完成 create/open/unlock/basic read-write。

## 3.1 已锁定开发决策

- 存储路线：MDBX 优先。
- 首个里程碑：MDBX iOS 技术验证。
- 最低系统版本：iOS 17+。
- 设备范围：iPhone 优先。
- Tiga 范围：首阶段只做 `Multi Type`。
- 第二阶段：本地 Vault。
- TestFlight 兼容范围：只做 MDBX。
- 备份：WebDAV 首版。
- AutoFill：首版较完整，包含 Credential Provider Extension、域名匹配、搜索、加密索引和 Extension 解锁。
- AutoFill 共享策略：App Group 加密索引。
- Passkey：后置。
- 授权/商业化：永久免费，不接 Plus、不接 IAP。
- Rust/Swift 桥接：UniFFI。
- 工程结构：模块化。

## 3.2 当前落地状态

- 已建立 `mdbx-ios-ffi` Rust crate，并通过 UniFFI 暴露最小技术验证 API。
- 已完成 iOS device/simulator XCFramework 构建脚本，输出 `MonicaMDBXGenerated.xcframework`。
- 已把 UniFFI Swift binding 接入 `MonicaMDBX` Swift package，并增加 Swift 友好的 wrapper。
- 已创建 `Monica.xcodeproj`，包含 SwiftUI App target、AutoFill Extension target 与 `MonicaTests` target。
- 已在 App Vault 页接入 MDBX round-trip 检查入口，用于验证 iOS 端 create/open/unlock/read-write。
- 已在 `MonicaStorage` 中建立 `LocalVaultRepository` / `LocalVaultEngine` 边界，让后续 App vault 管理通过 Storage 模块进入 MDBX。
- App Vault 页已接入 `LocalVaultRepository`，支持创建、打开和锁定本地 MDBX vault，并已完成密码条目、安全笔记、TOTP、银行卡和证件元数据的 create/list/search/edit/delete/restore/favorite 基础纵切；收藏状态写入加密 entry payload，不改 MDBX v1 schema，不进入 AutoFill index、secret snapshot 或 credential identities；各类型列表现在默认收藏优先，并支持 Favorites Only 筛选且可与搜索叠加；密码条目可用 `MonicaCore.PasswordGenerator` 生成默认 20 位随机密码并填入草稿；TOTP 可通过 `MonicaCore` 根据存储 seed 生成当前验证码，支持粘贴 `otpauth://` URI 或相机扫描 QR payload 导入到草稿表单，无效 QR 会显示可读错误并恢复扫描；列表和选中态会显示每秒刷新的验证码剩余时间。
- 已把 Keychain/LocalAuthentication 解锁接到 MDBX `security_key` 路线：手动解锁后注册本地 security key material，认证后用该 material 打开 remembered vault，不保存或恢复主密码。
- 已将主 App、AutoFill Extension、权限说明、基线状态和用户可读错误中文化；技术名、协议名、算法名和 bundle/service 标识继续保留英文。
- 已复用 Android launcher 图标生成 iOS AppIcon asset catalog，并接入 `Monica` target 的 Resources build phase；生成 PNG 已确认为不透明资源。
- 已通过 Rust smoke、SwiftPM 测试、iPhone 17 Pro simulator build 和 iOS simulator XCTest；最近一次完整 iPhone 17 Pro simulator XCTest 通过 53 个用例。

## 4. MVP 范围

### 首版包含

- 首次启动设置。
- 创建主密码、解锁和锁定。
- 使用 LocalAuthentication + Keychain 保护本地 security key material，并通过 MDBX `security_key` 实现快速解锁。
- 本地 vault 列表：
  - 密码
  - TOTP
  - 安全笔记
  - 银行卡
  - 身份/证件类元数据
- 新增、编辑、删除、收藏、搜索、回收站基础能力。
- TOTP 生成和二维码导入。
- 密码生成器。
- 手动导入和导出。
- WebDAV 备份和恢复。
- 基础 iOS AutoFill Credential Provider Extension，支持用户名/密码选择和填充。
- 使用已知样本 vault 和合成 fixture 做兼容性测试。

### 暂缓实现

- 完整 Bitwarden 双向同步。
- Passkey 创建和认证。
- 附件。
- 高级冲突管理 UI。
- 完整 MDBX 冲突管理和快照浏览 UI。
- iOS Widget。
- Apple Watch 应用。
- 通知栏式 TOTP 体验。
- Android IME 和 Accessibility 等无直接 iOS 等价物的能力。
- Monica Plus 授权、Apple IAP 或订阅。

## 5. 目标 iOS 架构

### App Target

- `Monica iOS App`
  - SwiftUI 主应用。
  - 负责主导航、设置、vault 管理、条目编辑和普通用户流程。

- `MonicaAutoFillExtension`
  - AuthenticationServices Credential Provider Extension。
  - 负责 AutoFill 列表、搜索、解锁门禁和 credential 返回。

- 后续可选：`MonicaShareExtension`
  - 用于接收导入文件或分享进来的 URL。

### Swift Package / 模块

- `MonicaCore`
  - 领域模型、条目校验、搜索模型、密码生成、TOTP。
  - 不依赖 UIKit 或 SwiftUI。

- `MonicaSecurity`
  - 主密码校验、KDF 策略、Keychain 访问、生物识别解锁 token、内存清理辅助。

- `MonicaStorage`
  - MDBX wrapper、迁移、Repository、查询 API。

- `MonicaMDBX`
  - 作为 Rust MDBX 的 UniFFI Swift wrapper。
  - 负责 FFI 边界、生成绑定和兼容测试入口。

- `MonicaSync`
  - WebDAV 客户端、备份包创建、恢复校验、重试和 backoff 策略。

- `MonicaUI`
  - 可复用 SwiftUI 组件和功能页面。

## 6. 数据和存储计划

### 战略方向

MDBX 应作为 Monica 的战略跨平台 vault 格式。仓库中已经有 Rust workspace 和正式规范文档，因此 iOS 应优先考虑：

1. 通过 UniFFI + XCFramework 复用 Rust MDBX 核心。
2. SwiftUI App 只通过 `MonicaMDBX` wrapper 访问 vault。
3. 不引入临时 SQLite/SQLCipher 作为首版主存储。

### 临时存储规则

如果 UniFFI 或 iOS 打包遇到阻塞，必须重新评审是否临时回退到 C ABI。不得未经确认改为 iOS 自建主存储。

## 7. 安全计划

### 核心原则

- 主密码永不存储。
- 原始 vault key 不写入 UserDefaults 或 App Group 明文存储。
- 生物识别只解开被包装的密钥，不能代替 vault 密钥本身。
- AutoFill Extension 只拿到匹配和填充所需的最小数据。
- 所有 App Group 共享文件都必须加密或降维成最小必要元数据。

### iOS 安全组件

- Keychain 保存被包装的密钥材料和解锁元数据。
- 可用时使用 Secure Enclave 做访问控制。
- LocalAuthentication 负责 Face ID / Touch ID 门禁。
- App Group 只用于共享加密 vault、加密搜索索引或 Extension 安全状态。

### 会话规则

- 主 App 和 Extension 有独立会话计时。
- Extension 解锁窗口应短。
- App 进入后台时立即遮挡敏感 UI，并清理临时明文值。
- 当前 iOS App 已实现 scene phase 基础策略：`inactive` 显示隐私遮罩，`background` 立即锁定并清理 App 会话中的敏感状态。
- 复制到剪贴板的内容应尽可能设置过期，并让用户感知。

## 8. AutoFill 计划

iOS AutoFill 必须作为独立 Extension 设计，不能把它当成主 App 里的一个 SwiftUI 页面。

职责：

- 通过 AuthenticationServices 维护 credential identities。
- vault 锁定时展示解锁流程。
- 搜索或筛选匹配账号。
- 将选中的用户名/密码返回给系统。
- 避免在 Extension 进程暴露不必要字段。

重要约束：

- Extension 生命周期短，内存受限。
- Extension 可能在主 App 未启动时被系统拉起。
- 共享数据需要通过 App Group。
- Extension 无法复制 Android overlay 或 IME 工作流。

## 9. Passkey 计划

Passkey 应等密码 AutoFill Extension 稳定后再做。

实现时必须围绕 iOS AuthenticationServices 和 relying-party identifier 重新设计。Android Passkey 记录可以通过兼容层映射，但 iOS 行为必须遵守 associated domains 和平台 Passkey 规则。

初始 Passkey 里程碑：

- 读取和展示已有 Passkey 元数据。
- 验证 relying-party ID 标准化。
- 增加兼容性测试。
- 之后再实现创建和认证流程。

## 10. 同步和备份计划

### MVP

- 手动 WebDAV 备份。
- 手动 WebDAV 恢复。
- 恢复前做备份完整性校验。
- 展示清晰恢复预览。
- 破坏性恢复前要求用户确认。

### 后续

- 使用 iOS 后台任务 API 做尽力而为的后台刷新。
- 冲突检测 UI。
- MDBX sync bundle 交换。
- WebDAV 稳定后再接入 OneDrive 或其他云服务。

iOS 版不能承诺 Android WorkManager 那种周期性后台同步体验。产品文案上应明确：后台同步是尽力而为，手动备份是一等能力。

## 11. Android 到 iOS 功能映射

| Android 区域 | iOS 方向 |
| --- | --- |
| Jetpack Compose UI | SwiftUI |
| Navigation Compose | NavigationStack / 类型化路由协调器 |
| Room database | MDBX wrapper |
| Android Keystore | Keychain + Secure Enclave |
| BiometricPrompt | LocalAuthentication |
| WorkManager | BGTaskScheduler，备份仍以前台优先 |
| AutofillService | Credential Provider Extension |
| Credential Provider / Passkey | AuthenticationServices |
| IME | 无直接等价物，MVP 不迁移 |
| Accessibility overlay assist | 无直接等价物，重设计或省略 |
| DataStore | 非敏感设置用 UserDefaults，敏感状态用 Keychain |
| Koin / 手动 DI | protocol-based dependency injection |
| WebDAV helper | 基于 URLSession 的 WebDAV 客户端 |
| KeePass / kotpass | 评估 Swift 原生库或共享 Rust 路线 |
| MDBX Rust workspace | 推荐通过 XCFramework 复用共享核心 |

## 12. 实施里程碑

### 里程碑 0：可行性验证

当前状态：模拟器路径已基本达成；仍需真机 device slice 实测。

- 将 Rust MDBX 编译到 iOS simulator 和真机。
- 通过 UniFFI 生成 Swift binding。
- 暴露最小 API：创建 vault、打开 vault、解锁、创建 project、在 project 下创建 entry、读回 entry。
- 创建空 Credential Provider Extension，确认 entitlement 和启动流程。
- 验证主 App 与 Extension 通过 App Group 读写加密测试数据。

退出标准：

- iOS 侧能通过 Swift/UniFFI 完成 MDBX create/open/unlock/project-entry read-write。
- 同一 fixture 能被 Rust CLI 或 Rust storage 测试读取。

### 里程碑 1：安全本地 Vault

当前状态：SwiftUI App 壳、create/open/lock 会话雏形、自动锁定和 Keychain/LocalAuthentication + MDBX `security_key` 解锁路径已开始落地；密码条目、安全笔记、TOTP、银行卡和证件元数据存储/编辑/收藏纵切已接入 App，列表已支持收藏优先和“只看收藏”筛选，TOTP 基础验证码生成、`otpauth://` URI 导入、相机 QR 扫描入口、扫描错误可读提示和每秒剩余时间刷新已接入 Core 和 App；主 App 和 AutoFill Extension 用户可见文案已中文化，iOS AppIcon 已复用 Android launcher 图标；完整安全状态机和真机签名验证尚未完成。

- SwiftUI App 壳。
- 首次启动设置。
- 主密码策略。
- Keychain-backed MDBX `security_key` unlock。
- 锁定和自动锁定。
- 基础加密存储。

退出标准：

- 可以创建、锁定、解锁并重新打开本地 vault。
- 单元测试覆盖密码校验和失败解锁行为。

### 里程碑 2：核心条目管理

- 密码、TOTP、笔记、银行卡和证件元数据。
- 搜索和收藏；当前 login/note/TOTP/card/identity 收藏 MVP、收藏优先排序和 Favorites Only 筛选已完成。
- 删除和恢复。
- 密码生成器。
- TOTP `otpauth://` URI 导入、相机二维码扫描入口、扫描错误提示和剩余秒数刷新基础已完成；真机相机权限和扫描画面待签名设备验证。

退出标准：

- 不依赖 AutoFill，用户也能完成日常凭据管理。

### 里程碑 3：备份和恢复

- 导出加密备份。
- 导入加密备份。
- WebDAV 上传和下载。
- 恢复预览和校验。

退出标准：

- 测试 vault 经过备份和恢复后，条目数量和关键字段一致。

### 里程碑 4：iOS AutoFill

- Credential identity 索引。
- Extension 锁定/解锁流程。
- credential 选择和填充。
- App Group 加密元数据。

退出标准：

- Safari 或 App 登录字段可以从 Monica 获取凭据。

### 里程碑 5：兼容性扩展

- KeePass 导入/导出或读写支持。
- 如果前面未使用 MDBX，此阶段完成 MDBX 完整支持。
- Android/iOS round-trip 测试。
- 冲突和快照 UI 基础。

退出标准：

- 同一个兼容性 fixture 可以在 Android、iOS 和 CLI 中一致打开。

### 里程碑 6：高级功能

- Passkey。
- 附件。
- Bitwarden 同步。
- OneDrive 或其他云服务。
- 高级安全分析。
- 如有必要，加入 Plus / 授权。

## 13. 测试策略

### 单元测试

- KDF 和密钥包装。
- TOTP 生成。
- 密码生成。
- URL / 域名标准化。
- 条目校验。
- 搜索排序。

### 集成测试

- vault 创建、打开、锁定、解锁。
- 备份和恢复。
- WebDAV fake server。
- App Group 数据共享。
- AutoFill Extension credential 选择。
- MDBX fixture round trip。

### 安全测试

- 解锁失败行为。
- 损坏 vault 处理。
- 恢复包篡改。
- vault 锁定时 Extension 访问。
- 剪贴板过期行为。

### 兼容性 Fixture

- 合成 Android Room export。
- 带 key file 的 KeePass `.kdbx` 样本。
- 各 Tiga 模式的 MDBX 样本 vault。
- 冲突、tombstone、snapshot 场景。

## 14. 产品和 UX 原则

- 使用 iOS 原生导航和设置习惯，不复制 Android 页面结构。
- 首次启动要短、清晰、能建立安全信任。
- 锁定状态必须明显。
- 不承诺 iOS 无法保证的后台同步。
- 恢复、删除等危险操作必须可预览，并尽可能可撤销。
- AutoFill 是工具界面：快速、聚焦、低文字、低摩擦。

## 15. 主要风险和缓解

| 风险 | 影响 | 缓解 |
| --- | --- | --- |
| MDBX FFI 不稳定或性能不足 | 阻塞共享格式 MVP | 提前做里程碑 0，并保留临时 SQLite 备选。 |
| AutoFill Extension 无法安全访问足够元数据 | AutoFill 体验差 | 设计加密/最小化身份索引，并尽早验证 App Group。 |
| iOS 后台同步不可靠 | 用户信任下降 | 把手动备份做成一等能力，后台备份标为尽力而为。 |
| 功能对齐压力扩大范围 | 首版延期 | 严格执行 MVP 验收标准。 |
| 安全模型与 Android 分叉 | vault 不兼容或不安全 | 先写兼容测试向量，再做高级同步。 |
| Passkey 平台规则差异 | 登录流程失败 | Passkey 延后到 AutoFill 和域名映射稳定之后。 |

## 16. 已关闭的早期决策

以下早期问题已经在开发路径决策中关闭：

1. iOS MVP 直接使用 MDBX，不使用临时 SQLite/SQLCipher 主存储。
2. 第一个公开 TestFlight 包含 MDBX 本地 vault、本地核心条目管理、WebDAV 和较完整 AutoFill。
3. Bitwarden 同步不进入首个公开 TestFlight。
4. KeePass 不进入首个公开 TestFlight。
5. Monica Plus / 授权 / Apple IAP 不进入 iOS 第一版，iOS 路线按永久免费推进。

## 17. 建议的立即下一步

当前建议从本地 Vault MVP 和真机签名验证两条线继续推进：

- 在真机上运行 `Monica` scheme 和 `MonicaTests`，确认 device slice 与签名配置。
- 在签名真机上验证 Keychain/LocalAuthentication + MDBX `security_key` 解锁路径，并补更完整的错误提示。
- 在真机上验证 TOTP 相机二维码扫描，并继续打磨扫描视觉细节。
- 继续打磨本地 Vault 多类型条目体验。
- 在真实 WebDAV 服务上做备份/恢复兼容测试。
