const https = require('https');
const http = require('http');

async function fetchUrl(url) {
  return new Promise((resolve, reject) => {
    const client = url.startsWith('https') ? https : http;
    const request = client.get(url, (response) => {
      let data = '';
      response.on('data', chunk => data += chunk);
      response.on('end', () => resolve({ status: response.statusCode, data }));
    });
    request.on('error', reject);
  });
}

async function testViewer() {
  try {
    console.log('Testing main viewer page...');
    const pageResponse = await fetchUrl('http://localhost:3000/');
    console.log(`Page status: ${pageResponse.status}`);
    console.log(`Page content length: ${pageResponse.data.length}`);
    console.log('Page snippet:', pageResponse.data.substring(0, 200));
    
    // Check for the game container
    if (pageResponse.data.includes('game-container')) {
      console.log('✅ Game container found in HTML');
    } else {
      console.log('❌ Game container NOT found in HTML');
    }
    
    // Test API through proxy
    console.log('\nTesting API proxy...');
    const apiResponse = await fetchUrl('http://localhost:3000/api/world_info');
    console.log(`API status: ${apiResponse.status}`);
    console.log(`API response: ${apiResponse.data.substring(0, 100)}`);
    
    // Test main script
    console.log('\nTesting main script...');
    const scriptResponse = await fetchUrl('http://localhost:3000/src/main-mvp.ts');
    console.log(`Script status: ${scriptResponse.status}`);
    console.log(`Script length: ${scriptResponse.data.length}`);
    
  } catch (error) {
    console.error('Error:', error.message);
  }
}

testViewer();
