function transformCallback(callback, once = false) {
	return window.__TAURI_INTERNALS__.transformCallback(callback, once)
}
async function invoke(cmd, args = {}) {
	return window.__TAURI_INTERNALS__.invoke(cmd, args)
}
async function invoke_result(cmd, args = {}) {
	return window.__TAURI_INTERNALS__.invoke(cmd, args)
}
function convertFileSrc(filePath, protocol = 'asset') {
	return window.__TAURI_INTERNALS__.convertFileSrc(filePath, protocol)
}
function isTauri() {
	return 'isTauri' in window && !!window.isTauri
}
export {
	invoke,
	invoke_result,
	convertFileSrc,
	transformCallback,
	isTauri,
}
