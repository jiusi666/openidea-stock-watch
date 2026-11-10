#!/usr/bin/env bash
# 首次推送用：读一次 GitHub Token，先用 API 校验，再保存并推送 main 与 v0.9.5。
# 用法：bash publish/push.sh
set -u
cd "$(dirname "$0")/.." || exit 1

ACCOUNT=jiusi666
REPO=openidea-stock-watch
PLAIN="https://github.com/${ACCOUNT}/${REPO}.git"

printf 'GitHub Token（classic，勾 public_repo 即可；输入不回显）: '
stty -echo 2>/dev/null
read -r TOKEN
stty echo 2>/dev/null
printf '\n'

if [ -z "${TOKEN}" ]; then
  printf '没有输入，已退出。\n'
  exit 1
fi

AUTHED="https://${ACCOUNT}:${TOKEN}@github.com/${ACCOUNT}/${REPO}.git"

printf '1/3 用 GitHub API 校验 token…\n'
CODE=$(curl -sS -o /dev/null -w '%{http_code}' --max-time 25 \
  -H "Authorization: Bearer ${TOKEN}" -H "Accept: application/vnd.github+json" \
  https://api.github.com/user 2>/dev/null || echo 000)
printf '  HTTP %s\n' "${CODE}"
case "${CODE}" in
  200) ;;
  401) printf '  token 无效或已过期，重新生成一个。\n'; exit 1 ;;
  403) printf '  token 权限不足，确认勾了 public_repo。\n'; exit 1 ;;
  000) printf '  连不上 api.github.com，检查网络或代理后重试。\n'; exit 1 ;;
  *)   printf '  校验没通过（HTTP %s）。\n' "${CODE}"; exit 1 ;;
esac

printf '2/3 保存到 Windows 凭据管理器（以后 push 不再询问）…\n'
if printf 'protocol=https\nhost=github.com\nusername=%s\npassword=%s\n\n' "${ACCOUNT}" "${TOKEN}" \
    | git credential approve >/dev/null 2>&1; then
  printf '  已保存\n'
else
  printf '  跳过（不影响本次推送）\n'
fi

printf '3/3 推送代码与标签…\n'
if git push -u origin main && git push origin v0.9.5; then
  :
else
  printf '  origin 推送失败，改用带凭据的一次性地址重试…\n'
  git push "${AUTHED}" main || exit 1
  git push "${AUTHED}" v0.9.5 || exit 1
  git fetch origin >/dev/null 2>&1 && git branch --set-upstream-to=origin/main main >/dev/null 2>&1
fi

printf '\n完成：https://github.com/%s/%s\n' "${ACCOUNT}" "${REPO}"
printf '下一步：Releases → 选 v0.9.5 → 附上 dist/jiusi.stock-watch-0.9.5.oiplugin → 发布\n'
