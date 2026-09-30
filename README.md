# NR Manager

Gerenciador desktop (Windows 11) para o mod **DLSS 5 Neural Rendering on AMD**
(`danielblnc/DLSS-NR-on-AMD`), com visual inspirado no AMD Radeon Software
Adrenalin / NVIDIA App.

O app **não redistribui** nenhum arquivo do mod nem o `nvngx_dlssnr.dll`. Ele
sempre baixa o `dlssnr_on_amd_setup.exe` da release oficial no GitHub (na máquina
do usuário) e pede uma única vez o caminho do `nvngx_dlssnr.dll`, que é fornecido
pelo próprio usuário (obtido de um jogo com DLSS 5).

> Projeto não oficial, sem vínculo com NVIDIA/AMD nem com o autor do mod.

---

## Recursos

- **Biblioteca de jogos**: detecção automática de Steam (múltiplas bibliotecas),
  Epic Games, GOG e Xbox/Game Pass; inclusão manual por pasta ou `.exe`; capas da
  Steam; localização heurística do executável principal (inclui subpastas como
  `bin\x64`).
- **Compatibilidade**: `compat.json` editável (Cyberpunk 2077 e GTA V Enhanced já
  como testados), heurísticas DX12/FSR, selos *Compatível / Provável / Não testado
  / Incompatível* e filtros.
- **GPU**: detecção via WMI (PowerShell CIM) e aviso se não for RDNA3/RDNA4.
- **Versões**: consulta à API pública do GitHub (ETag, cache, rate limit, offline),
  changelog formatado, download do asset oficial, atualização em massa por jogo,
  rollback (versão fixada) e importação de setup local (builds do Discord).
- **Aplicar/Desaplicar reversível** por jogo, com **backup antes de qualquer
  alteração** e manifesto de desinstalação.
- **Reparar/verificar integridade** por hash, detecção de divergência após
  atualização do jogo.
- **Aviso de anti-cheat** (EAC, BattlEye, Vanguard etc.).
- Configurações, logs persistentes e tema escuro.

## Stack

**Tauri 2 (Rust) + React + TypeScript + Tailwind CSS**, com Lucide Icons e Zustand.
Escolhemos Tauri em vez de Electron por ser nativo, muito menor (~poucos MB) e por
ter acesso direto ao sistema de arquivos/registro/processos no backend Rust — ideal
para snapshots, backups e detecção de jogos.

---

## Pré-requisitos

Para **rodar em modo dev** ou **compilar** você precisa de:

1. **Windows 11** (x64).
2. **Node.js 18+** e npm — https://nodejs.org
3. **Rust (stable)** via [rustup](https://rustup.rs) — inclui `cargo`.
4. **Microsoft Edge WebView2 Runtime** (já presente no Windows 11).
5. **Visual Studio Build Tools** com o workload *Desktop development with C++*
   (C++ build tools + Windows SDK), exigido pelo Rust/MSVC no Windows.

O script `build.bat` verifica esses itens e orienta caso falte algo.

## Modo dev

```bat
npm install
npm run tauri:dev
```

## Build (instalador NSIS)

```bat
build.bat
```

ou manualmente:

```bat
npm install
npm run tauri:build
```

Artefatos gerados em:

- Instalador: `src-tauri\target\release\bundle\nsis\NR Manager_*_x64-setup.exe`
- Executável portátil: `src-tauri\target\release\NR Manager.exe`
  (rode `npm run bundle:portable` para gerar apenas o `.exe`).

---

## Onde ficam os dados

Tudo é local, em `%APPDATA%\com.nrmanager.app`:

| Caminho | Conteúdo |
|---|---|
| `config.json` | Configurações do app |
| `library.json` | Biblioteca de jogos |
| `compat.json` | Lista de compatibilidade (editável) |
| `manifests\<jogo>.json` | Cópias dos manifestos de instalação |
| `backups\<jogo>\...` | Backups dos arquivos originais sobrescritos |
| `logs\nrmanager.log` | Log do aplicativo |
| `%LOCALAPPDATA%\com.nrmanager.app\cache\releases` | Setups baixados/importados |

`config.json` e `compat.json` de exemplo estão na raiz do projeto; o app semeia o
`compat.json` a partir do arquivo embutido na primeira execução.

---

## Como funciona aplicar / desaplicar

O instalador do mod é um `setup.exe` interativo (não documentamos flags de linha
de comando). Por isso o app **descobre o que ele altera em tempo de execução**,
comparando a pasta antes e depois:

### Aplicar

1. **Pré-requisitos**: executável localizado, pasta existente, `nvngx_dlssnr.dll`
   configurado, jogo fechado (senão o arquivo fica em uso).
2. **Snapshot "antes"**: varredura recursiva da pasta do mod (onde fica o `.exe`)
   registrando caminho relativo, tamanho, `mtime` e **SHA-256**. Arquivos gigantes
   (acima do limite configurável, padrão 512 MB) não recebem hash — evita copiar
   arquivos de dados enormes (ex.: GTA V).
3. **Backup**: todos os arquivos pequenos do snapshot são copiados para
   `backups\<jogo>\` (mantendo a estrutura). Isso permite restaurar os originais
   mesmo que o instalador sobrescreva arquivos existentes.
4. **Preparação**: copia `dlssnr_on_amd_setup.exe` e `nvngx_dlssnr.dll` para a
   pasta do jogo (exatamente o fluxo manual documentado pelo mod).
5. **Execução**: abre o instalador **elevado** (`Start-Process -Verb RunAs -Wait`)
   e aguarda o término. O app acompanha o término e lê o exit code.
6. **Snapshot "depois"** e **diff**:
   - `created_files` = existem só depois (novos);
   - `modified_files` = mudaram (hash ou tamanho/`mtime`);
   - `deleted_files` = existiam antes e sumiram;
   - `created_dirs` = diretórios novos.
7. **Manifesto** `nrmanager.manifest.json` gravado **na pasta do jogo** e uma
   cópia em `manifests\<jogo>.json`.
8. Falha do instalador (exit code ≠ 0) → **rollback automático**: remove criados,
   restaura originais do backup e apaga o manifesto.

### Desaplicar

Usa **apenas o manifesto**:

1. Remove os `created_files`.
2. Restaura `modified_files` e `deleted_files` a partir do backup.
3. Remove `created_dirs` (somente se estiverem vazias).
4. Apaga os manifestos e o backup.

**Nada fora do manifesto é apagado.** Se o jogo for atualizado e sobrescrever/
remover arquivos do mod, a verificação por hash marca "divergência" e o app
oferece **Reaplicar**.

---

## Suposições sobre o instalador do mod

- O `setup.exe` é **interativo**; assumimos que ele não expõe flags silenciosas.
  O app o executa com elevação, aguarda o usuário concluir e usa o **exit code**
  para decidir sucesso/rollback. Há um campo opcional de "argumentos silenciosos"
  nas Configurações para builds que venham a suportá-los.
- O instalador precisa do `nvngx_dlssnr.dll` **na mesma pasta** (conforme README
  oficial); por isso copiamos o DLL escolhido para a pasta do jogo antes de rodar.
- Como não conhecemos os nomes exatos dos arquivos instalados, a detecção é feita
  por **snapshot antes/depois** (arquivos + hash). O nome do arquivo de
  configuração do mod (`*.ini`/`*.cfg`/`*.json`) é procurado heurísticamente.
- As versões e assets são lidos da **API pública do GitHub**; o `nvngx_dlssnr.dll`
  nunca é baixado pelo app.

## Aviso legal / segurança

- Faça backup e use por sua conta e risco. DLLs modificadas podem ser bloqueadas
  por **anti-cheat** — use apenas em jogos **offline/single-player**.
- O app nunca redistribui arquivos de terceiros e só toca em arquivos que ele
  mesmo registrou no manifesto.
