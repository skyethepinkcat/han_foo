vim.lsp.config("rust_analyzer", {
	settings = {
		['rust-analyzer'] = {
			cargo = {
				target = "wasm32-unknown-unknown";
			},
		},
	},
})
vim.lsp.enable("rust_analyzer")
vim.lsp.enable("cssls")
