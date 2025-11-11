#!/bin/bash
echo "🔄 Restarting AudiobookShelf stack..."

# Stop everything
docker stop audiobookshelf nginx-proxy-manager 2>/dev/null

# Wait
sleep 3

# Start
docker start nginx-proxy-manager
sleep 2
docker start audiobookshelf

# Check status
echo ""
echo "📊 Status:"
docker ps | grep -E "audiobookshelf|nginx"

echo ""
echo "🌐 Try accessing: http://secretlibrary.org/audiobookshelf/"
echo "   Or locally: http://localhost:13378/"
