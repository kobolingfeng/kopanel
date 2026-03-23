# Widget 打包发布注意事项

## 证书签名

### 重要：必须使用正确的证书签名

系统中可能存在多个 `CN=KoPanel` 证书，**必须使用与 `.cer` 文件匹配的证书**进行签名。

查看当前证书：
```powershell
Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Subject -like "*KoPanel*" } | Format-List Subject, Thumbprint, NotAfter
```

查看 .cer 文件的 Thumbprint：
```powershell
$cert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2("widget\KoPanelWidget.cer")
Write-Host "Thumbprint: $($cert.Thumbprint)"
```

### 正确的签名命令

使用 `.cer` 文件对应的证书 Thumbprint 签名：
```powershell
# 先确认 .cer 的 Thumbprint，然后用该 Thumbprint 签名
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe" sign /fd SHA256 /sha1 <THUMBPRINT> /t http://timestamp.digicert.com "KoPanelWidget_x.x.x.x_x64.msix"
```

当前正确的 Thumbprint：`06CC2AB8CF63C588C73C6D2411A5A18AE40531D8`（有效期至 2031/1/26）

### 验证签名
```powershell
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe" verify /pa "path\to\file.msix"
```

---

## 版本号排序问题

### 问题
`publish.bat` 使用字符串排序查找最新 Widget，导致 `1.0.0.10` 排在 `1.0.0.9` 之前（因为 "1" < "9"）。

### 解决方案
已修复为使用 `[version]` 类型排序：
```powershell
Get-ChildItem "widget\AppPackages\*_x64_Test\*.msix" | 
  ForEach-Object { 
    $_ | Add-Member -NotePropertyName Ver -NotePropertyValue ([version]($_.Name -replace '.*_(\d+\.\d+\.\d+\.\d+)_.*','$1')) -PassThru 
  } | 
  Sort-Object Ver -Descending | 
  Select-Object -First 1
```

---

## 编译 Widget

### 使用 MSBuild 编译
```powershell
# 需要在 VS Developer PowerShell 环境中运行
$vsPath = & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -property installationPath
& "$vsPath\Common7\Tools\Launch-VsDevShell.ps1" -Arch amd64 -HostArch amd64

msbuild KoPanelWidget.csproj /p:Configuration=Release /p:Platform=x64 /p:AppxBundle=Never /p:AppxPackageSigningEnabled=false /p:UapAppxPackageBuildMode=SideloadOnly
```

### 版本号更新
修改 `widget\Package.appxmanifest` 中的 Version：
```xml
<Identity
  Name="KoPanel.Widget"
  Publisher="CN=KoPanel"
  Version="1.0.0.10" />
```

---

## 完整打包流程

### 随主程序一起发布（推荐）

运行 `scripts\publish.bat`，会自动处理 Widget：
- 查找最新的 Widget MSIX
- 打包为 zip（包含 msix + cer + 安装证书.bat）
- 更新 update.json 中的 Widget 信息
- 上传所有文件到 R2

### 单独更新 Widget

1. **更新版本号** - `widget\Package.appxmanifest`
2. **编译** - 使用 MSBuild
3. **运行** `scripts\publish_widget.bat`
   - 自动使用长期证书签名
   - 自动打包（msix + cer + 安装证书.bat）
   - 自动上传到 R2
4. **手动更新 update.json**：
   ```powershell
   # 下载当前 update.json
   rclone copy r2:altv/kopanel/update.json .
   
   # 修改 Widget 信息
   $json = Get-Content update.json | ConvertFrom-Json
   $json.widget_url = "https://cdn.kobo07.cn/kopanel/KoPanelWidget_x.x.x.x.zip"
   $json.widget_version = "x.x.x.x"
   $json.widget_size = <文件大小>
   $json | ConvertTo-Json -Depth 10 | Out-File update.json -Encoding UTF8
   
   # 上传更新后的 update.json 和下载页
   rclone copy update.json r2:altv/kopanel
   rclone copy downloadpage r2:altv/kopanel
   ```

---

## 常见问题

### 安装证书后仍无法安装 MSIX
**原因**：签名用的证书与 .cer 文件不匹配  
**解决**：用 .cer 对应的证书重新签名

### Widget 下载的是旧版本
**原因**：publish.bat 字符串排序问题  
**解决**：已修复，使用 [version] 类型排序

### dotnet build 失败
**原因**：UWP 项目需要使用 MSBuild，不能用 dotnet CLI  
**解决**：使用 VS Developer PowerShell + MSBuild

### 检查更新失败 / JSON 解析错误
**原因**：update.json 文件包含 UTF-8 BOM（`EF BB BF`），Rust 的 JSON 解析器无法正确解析  
**解决**：不要使用 PowerShell 的 `Out-File -Encoding UTF8`，改用：
```powershell
[System.IO.File]::WriteAllText("path\to\update.json", $jsonContent, (New-Object System.Text.UTF8Encoding $false))
```

**验证方法**：
```powershell
$bytes = [System.IO.File]::ReadAllBytes("update.json")
Write-Host "前 3 字节: $($bytes[0]), $($bytes[1]), $($bytes[2])"
# 应该是 123 ("{" 的 ASCII 码)，不应该是 239, 187, 191 (BOM)
```

---

## 相关文件路径

- Widget 项目：`widget\KoPanelWidget.csproj`
- Manifest：`widget\Package.appxmanifest`
- 证书文件：`widget\KoPanelWidget.cer`
- 安装脚本：`widget\安装证书.bat`
- 编译输出：`widget\AppPackages\KoPanelWidget_x.x.x.x_x64_Test\`
- 发布脚本：`scripts\publish.bat`
