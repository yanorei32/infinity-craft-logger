# Infinite Craft Logger

## Examples

### Server

```
./infinite-craft-logger # default port usage (127.0.0.1:8888)
./infinite-craft-logger 127.0.0.1:8080 # specific port usage
```

### Client

```javascript
const original_fetch = window.fetch;

const TARGET = "http://localhost:8888/api/infinite-craft/recipe";

function my_fetch(url) {
    if (url.startsWith('https://neal.fun/api/infinite-craft/pair')) {
        const parser = new URL(url);
        const first = parser.searchParams.get('first');
        const second = parser.searchParams.get('second');

        original_fetch(url).then(d => d.json()).then(response => {
            const result = response.result;
            const emoji = response.emoji;

            original_fetch(TARGET, {
                method: 'post',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    first: first,
                    second: second,
                    result: result,
                    emoji: emoji,
                })
            }).then(response => response.text()).then(console.log);
        });
    }


    return original_fetch(url);
}

window.fetch = my_fetch;
```
