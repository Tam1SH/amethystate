async function emit(event, payload) {
	return window.__TAURI_INTERNALS__.invoke('tauri', {
		__tauriModule: 'Event',
		message: {
			cmd: 'emit',
			event,
			payload: JSON.stringify(payload)
		}
	})
}
async function emitTo(target, event, payload) {
	return window.__TAURI_INTERNALS__.invoke('tauri', {
		__tauriModule: 'Event',
		message: {
			cmd: 'emitTo',
			target,
			event,
			payload: JSON.stringify(payload)
		}
	})
}
async function listen(event, handler, options) {
	return window.__TAURI_INTERNALS__.listen(event, handler, options)
}
async function once(event, handler, options) {
	return window.__TAURI_INTERNALS__.once(event, handler, options)
}
export { emit, emitTo, listen, once }
