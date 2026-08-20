# Registro da auditoria de dependências — 15/08/2026

O experimento isolado com dependência Unix usando `backend-winit-x11` abriu a UI real em Xvfb e carregou `/tmp` sem regressão visual; o alvo Windows `x86_64-pc-windows-gnu` também compilou com a dependência Windows usando `backend-winit`. A árvore normal do experimento caiu de 1275 para 1140 nós no Linux e manteve 892 nós no Windows.

A alteração foi aplicada ao projeto principal em `Cargo.toml` após validação. No estado aplicado, a árvore normal Linux do host caiu de 1047 para 913 nós; no alvo específico Linux, de 1275 para 1140; o binário Linux release caiu de 18.002.864 para 16.128.776 bytes; e o clean build Linux medido caiu de 149,09 s para 131,63 s. O alvo Windows manteve 892 nós e o binário release mediu 12.488.704 bytes antes e depois.

O relatório completo, com a matriz de dependências, decisões, features, duplicações transitivas, riscos e validação, está em [`./DEPENDENCY_AUDIT.md`](././DEPENDENCY_AUDIT.md).
