// ==UserScript==
// @name         Infinite Craft Logger SCHEME://HOST
// @namespace    https://github.com/yanorei32/infinite-craft-logger
// @version      0.1.0
// @description  Collect the recipe
// @author       yanorei32
// @match        https://neal.fun/infinite-craft/
// @icon         https://www.google.com/s2/favicons?sz=64&domain=neal.fun
// @updateURL    SCHEME://HOST/api/infinite-craft/client-script.user.js?token=TOKEN
// @grant        none
// @run-at       document-start
// ==/UserScript==

const original_fetch = window.fetch;

function my_fetch(url) {
	if (url.startsWith('https://neal.fun/api/infinite-craft/pair')) {
		const parser = new URL(url);
		const first = parser.searchParams.get('first');
		const second = parser.searchParams.get('second');

		original_fetch(url).then(d => d.json()).then(response => {
			const result = response.result;
			const emoji = response.emoji;
			original_fetch('SCHEME://HOST/api/infinite-craft/recipe', {
				method: 'post',
				headers: {
					'Authorization': AUTH ? 'Bearer TOKEN' : undefined,
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					first: first, second: second, result: result, emoji: emoji,
				})
			}).then(response => response.text()).then(console.log);
		});
	}

	return original_fetch(url);
}

window.fetch = my_fetch;
