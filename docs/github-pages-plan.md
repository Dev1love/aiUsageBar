# GitHub Pages + Releases — план публикации VibeUsageBar

Создано 2026-05-12. **Статус:** к обсуждению и реализации в следующей сессии.

## Цель

Превратить публичный репозиторий `Dev1love/aiUsageBar` в полноценную точку
дистрибуции:

1. **Лендинг** на `github.io` (или кастомном домене) — куда указывать ссылку
   тем, кому интересен проект.
2. **Скачиваемый билд** — стабильная ссылка на `.dmg` / `.app.tar.gz` для
   v0.3.0 и следующих версий.

## Что у нас есть из ресурсов

- `README.md` — короткое описание, requirements, install
- `docs/post-vibeusagebar.md` — длинная статья на русском, готовая для
  публикации (untracked, не запушено)
- Скриншоты иконки + попапа — из этой сессии есть несколько в чате
- Bundled `.app` собирается через `npx tauri build`

## Опции лендинга (от простого к сложному)

### Вариант A — README как Pages (15 минут)
Settings → Pages → Source: `main` branch, `/` root, тема Jekyll по умолчанию.
GitHub отрендерит `README.md` как стартовую страницу. Никакого кода писать
не надо. URL: `https://dev1love.github.io/aiUsageBar/`.

**Минусы:** скучный вид (хотя темы выбираются). README сейчас сухой
техдок, не маркетинг.

### Вариант B — `docs/` папка как Pages (30-60 минут)
Settings → Pages → Source: `main` branch, `/docs` folder. Положить туда
`index.md` (можно перенести/адаптировать `post-vibeusagebar.md`),
`_config.yml` с темой, опционально картинки в `docs/assets/`.

Получаем красивее, чем просто README, и весь маркетинговый текст в одном
месте.

### Вариант C — кастомный SSG через Actions (3-5 часов)
Astro / Eleventy / Hugo / просто HTML. Workflow собирает на push в main,
деплоит в `gh-pages` branch.

Имеет смысл если хочется анимаций, демо-видео встраивать, кастомный
дизайн. Для статус-бара пока overkill.

**Рекомендация: B.** README остаётся техдокументацией для разработчиков,
`docs/index.md` (на базе `post-vibeusagebar.md`) — для пользователей.

## Опции дистрибуции бинаря

### Вариант 1 — Release вручную с локальным билдом (30 минут)
```bash
git tag v0.3.0
git push --tags
npx tauri build  # производит .dmg в src-tauri/target/release/bundle/dmg/
gh release create v0.3.0 \
  --title "v0.3.0 — macOS 26 + system monitor" \
  --notes-file release-notes-v0.3.0.md \
  src-tauri/target/release/bundle/dmg/VibeUsageBar_0.3.0_aarch64.dmg
```

**Минусы:** каждый релиз собирается на твоей машине. Если нужен Intel
билд — отдельная команда (`--target x86_64-apple-darwin`).

### Вариант 2 — GitHub Actions auto-build (1-2 часа)
`.github/workflows/release.yml` с триггером `on: push: tags: [v*]`:
- runs-on: macos-latest
- Кэширует cargo + npm
- Запускает `npx tauri build`
- Аплоадит DMG в release через `softprops/action-gh-release@v2`

Можно сразу собирать обе архитектуры (arm64 + x86_64) через matrix.

**Минусы:** GitHub Actions на macOS-раннерах тратит минуты тарифного плана
быстрее (множитель 10x для приватных репо, но public — бесплатно).

### Подводный камень: Gatekeeper

Без подписи Apple Developer (\$99/год) и нотаризации:
- При первом запуске macOS покажет «cannot be opened because the developer
  cannot be verified»
- Юзеру нужно: right-click → Open → Open anyway
- В README надо добавить эту инструкцию

С подписью и нотаризацией:
- Билд проходит без предупреждений
- Workflow усложняется: нужны секреты с сертификатом, app-specific password,
  notarization API key

**Рекомендация для старта: Вариант 1 без нотаризации.** Скачивающие — в
большинстве технические юзеры, разберутся с right-click → Open. Когда (и
если) аудитория вырастет — нотаризация.

## План «минимального запуска» на следующую сессию

Час работы, без Apple Developer:

1. Подготовить `release-notes-v0.3.0.md` — выжимка changelog от v0.2.0 до
   v0.3.0 + macOS 26 fix.
2. `git tag v0.3.0 && git push --tags`
3. `npx tauri build` → получить DMG
4. `gh release create v0.3.0 ...` — прикрепить DMG
5. Включить GitHub Pages (Вариант B): создать `docs/index.md` на базе
   `post-vibeusagebar.md`, добавить ссылку на release.
6. В README добавить:
   - ссылку на Pages
   - ссылку на скачивание DMG (latest release)
   - параграф «First launch on unsigned build» с инструкцией right-click → Open
7. Обновить memory aiusagebar.md — указать URL лендинга.

## Опционально на следующих итерациях

- Actions workflow для автобилда на тег (Вариант 2)
- Скриншоты в README/Pages (попап, иконка в menubar)
- Apple Developer + нотаризация
- Universal binary (arm64 + x86_64)
- Кастомный домен на Pages (e.g. `vibeusagebar.app`)
- Sparkle-style auto-update (потребует подписи + сервер с appcast)
