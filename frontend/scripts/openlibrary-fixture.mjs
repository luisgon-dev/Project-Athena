import { createServer } from 'node:http';

const server = createServer((req, res) => {
	const url = new URL(req.url ?? '/', 'http://127.0.0.1:5001');

	if (url.pathname === '/search.json') {
		return replyJson(res, {
			docs: [
				{
					key: '/works/OL82563W',
					title: 'The Hobbit',
					author_name: ['J.R.R. Tolkien'],
					first_publish_year: 1937,
					cover_i: null,
					subject: ['Fantasy', 'Middle-earth'],
					edition_count: 12
				}
			]
		});
	}

	if (url.pathname === '/works/OL82563W.json') {
		return replyJson(res, {
			key: '/works/OL82563W',
			title: 'The Hobbit',
			description: 'A fantasy adventure in Middle-earth.',
			subjects: ['Fantasy', 'Middle-earth'],
			covers: [],
			authors: [{ author: { key: '/authors/OL26320A' } }],
			first_publish_date: '1937'
		});
	}

	if (url.pathname === '/authors/OL26320A.json') {
		return replyJson(res, { name: 'J.R.R. Tolkien' });
	}

	if (url.pathname.startsWith('/b/id/')) {
		res.writeHead(404);
		res.end();
		return;
	}

	res.writeHead(404, { 'content-type': 'application/json' });
	res.end(JSON.stringify({ error: `No fixture for ${url.pathname}` }));
});

server.listen(5001, '127.0.0.1', () => {
	console.log('openlibrary fixture listening on 127.0.0.1:5001');
});

function replyJson(res, payload) {
	res.writeHead(200, { 'content-type': 'application/json' });
	res.end(JSON.stringify(payload));
}

