import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { FileAudio, RefreshCw, Wrench, Settings, Upload, UploadCloud, Save, ChevronRight, ChevronDown, Folder, Book, Files, Clock, Zap, FileSearch, AlertCircle } from 'lucide-react';
import { RawTagInspector } from './components/RawTagInspector';

function App() {
  const [activeTab, setActiveTab] = useState('scanner');
  const [config, setConfig] = useState(null);
  const [groups, setGroups] = useState([]);
  const [expandedGroups, setExpandedGroups] = useState(new Set());
  const [scanning, setScanning] = useState(false);
  const [writing, setWriting] = useState(false);
  const [pushing, setPushing] = useState(false);
  const [selectedFiles, setSelectedFiles] = useState(new Set());
  const [selectedGroup, setSelectedGroup] = useState(null);
  const [showTagInspector, setShowTagInspector] = useState(false);
  const [fileStatuses, setFileStatuses] = useState({});

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const cfg = await invoke('get_config');
      setConfig(cfg);
    } catch (error) {
      console.error('Failed to load config:', error);
    }
  };

  const saveConfig = async (newConfig) => {
    try {
      await invoke('save_config', { config: newConfig });
      setConfig(newConfig);
      alert('Settings saved!');
    } catch (error) {
      console.error('Failed to save config:', error);
      alert('Failed to save settings');
    }
  };

  const handleScan = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: true,
      });
      
      if (!selected) return;
      
      const paths = Array.isArray(selected) ? selected : [selected];
      setScanning(true);
      setFileStatuses({});
      
      const result = await invoke('scan_library', { paths });
      setGroups(result.groups);
      
      if (result.groups.length > 0) {
        setSelectedGroup(result.groups[0]);
      }
      
      setScanning(false);
    } catch (error) {
      console.error('Scan failed:', error);
      setScanning(false);
      alert('Scan failed: ' + error);
    }
  };

  const handleWrite = async () => {
    if (selectedFiles.size === 0) {
      alert('No files selected');
      return;
    }

    try {
      setWriting(true);

      const filesMap = {};
      groups.forEach(group => {
        group.files.forEach(file => {
          filesMap[file.id] = {
            path: file.path,
            changes: file.changes
          };
        });
      });
      
      const result = await invoke('write_tags', { 
        request: {
          file_ids: Array.from(selectedFiles),
          files: filesMap,
          backup: config.backup_tags
        }
      });

      const newStatuses = { ...fileStatuses };
      Array.from(selectedFiles).forEach(fileId => {
        const hasError = result.errors.some(e => e.file_id === fileId);
        newStatuses[fileId] = hasError ? 'failed' : 'success';
      });
      setFileStatuses(newStatuses);
      
      setWriting(false);
      
      if (result.failed > 0) {
        alert(`Written: ${result.success} files\nFailed: ${result.failed} files`);
      } else {
        alert(`Successfully wrote tags to ${result.success} files!`);
        setSelectedFiles(new Set());
      }
    } catch (error) {
      console.error('Write failed:', error);
      setWriting(false);
      alert('Write failed: ' + error);
    }
  };

  const handlePush = async () => {
    const successfulFileIds = Array.from(selectedFiles).filter(id => fileStatuses[id] === 'success');
    
    if (successfulFileIds.length === 0) {
      alert('No successfully written files to push. Please write tags first.');
      return;
    }

    const skippedCount = selectedFiles.size - successfulFileIds.length;
    if (skippedCount > 0) {
      if (!confirm(`Push ${successfulFileIds.length} successfully written files?\n(Skipping ${skippedCount} failed/unwritten files)`)) {
        return;
      }
    }

    try {
      setPushing(true);
      
      const items = [];
      groups.forEach(group => {
        group.files.forEach(file => {
          if (successfulFileIds.includes(file.id)) {
            items.push({
              path: file.path,
              metadata: group.metadata
            });
          }
        });
      });

      const result = await invoke('push_abs_updates', { request: { items } });
      setPushing(false);

      let message = `Updated ${result.updated || 0} item${result.updated === 1 ? '' : 's'} in AudiobookShelf.`;

      if (result.unmatched && result.unmatched.length > 0) {
        message += `\nCould not find matches for: ${result.unmatched.slice(0, 5).join(', ')}${
          result.unmatched.length > 5 ? '…' : ''
        }`;
      }

      if (result.failed && result.failed.length > 0) {
        const failures = result.failed
          .slice(0, 5)
          .map((f) => `${f.path}${f.reason ? ` (${f.reason})` : ''}`)
          .join(', ');
        message += `\nFailed to update: ${failures}${result.failed.length > 5 ? '…' : ''}`;
      }

      alert(message);
    } catch (error) {
      console.error('Push to AudiobookShelf failed:', error);
      setPushing(false);
      alert('Failed to push updates: ' + error);
    }
  };

  const toggleGroup = (groupId) => {
    const newExpanded = new Set(expandedGroups);
    if (newExpanded.has(groupId)) {
      newExpanded.delete(groupId);
    } else {
      newExpanded.add(groupId);
    }
    setExpandedGroups(newExpanded);
  };

  const selectAllInGroup = (group, checked) => {
    const newSelected = new Set(selectedFiles);
    group.files.forEach(file => {
      if (checked) {
        newSelected.add(file.id);
      } else {
        newSelected.delete(file.id);
      }
    });
    setSelectedFiles(newSelected);
  };

  const getGroupIcon = (type) => {
    if (type === 'single') return <Book className="w-4 h-4" />;
    if (type === 'series') return <Folder className="w-4 h-4" />;
    return <Files className="w-4 h-4" />;
  };

  const getFileStatusIcon = (fileId) => {
    const status = fileStatuses[fileId];
    if (status === 'success') return <span className="text-green-600 font-bold">✓</span>;
    if (status === 'failed') return <span className="text-red-600 font-bold">✗</span>;
    return null;
  };

  const getSuccessCount = () => {
    return Array.from(selectedFiles).filter(id => fileStatuses[id] === 'success').length;
  };

  const getFailedCount = () => {
    return Array.from(selectedFiles).filter(id => fileStatuses[id] === 'failed').length;
  };

  const testConnection = async () => {
    try {
      const result = await invoke('test_abs_connection', { config });
      alert(result.message);
    } catch (error) {
      alert('Connection test failed: ' + error);
    }
  };

  if (!config) {
    return (
      <div className="h-screen flex items-center justify-center">
        <div className="text-gray-500">Loading...</div>
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      <header className="bg-white border-b border-gray-200 px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <FileAudio className="w-8 h-8 text-red-600" />
            <h1 className="text-2xl font-bold text-gray-900">Audiobook Tagger</h1>
          </div>
          <div className="flex items-center gap-4">
            <button onClick={handleScan} disabled={scanning} className="btn btn-primary flex items-center gap-2">
              <RefreshCw className={`w-4 h-4 ${scanning ? 'animate-spin' : ''}`} />
              {scanning ? 'Scanning...' : 'Scan Library'}
            </button>
            {selectedFiles.size > 0 && (
              <button onClick={handleWrite} disabled={writing} className="btn btn-primary flex items-center gap-2">
                <Save className="w-4 h-4" />
                {writing ? 'Writing...' : `Write ${selectedFiles.size} Files`}
              </button>
            )}
            {getSuccessCount() > 0 && (
              <button
                onClick={handlePush}
                disabled={pushing}
                className="btn btn-primary flex items-center gap-2"
              >
                <UploadCloud className={`w-4 h-4 ${pushing ? 'animate-pulse' : ''}`} />
                {pushing ? 'Pushing…' : `Push ${getSuccessCount()} to ABS`}
              </button>
            )}
            <button
              onClick={() => setShowTagInspector(true)}
              className="btn btn-secondary flex items-center gap-2"
            >
              <FileSearch className="w-4 h-4" />
              Inspect Tags
            </button>
          </div>
        </div>
      </header>

      <nav className="bg-white border-b border-gray-200 px-6">
        <div className="flex gap-1">
          <button
            onClick={() => setActiveTab('scanner')}
            className={`px-4 py-2 font-medium transition-colors ${
              activeTab === 'scanner'
                ? 'text-red-600 border-b-2 border-red-600'
                : 'text-gray-600 hover:text-gray-900'
            }`}
          >
            <div className="flex items-center gap-2">
              <RefreshCw className="w-4 h-4" />
              Scanner
            </div>
          </button>
          <button
            onClick={() => setActiveTab('maintenance')}
            className={`px-4 py-2 font-medium transition-colors ${
              activeTab === 'maintenance'
                ? 'text-red-600 border-b-2 border-red-600'
                : 'text-gray-600 hover:text-gray-900'
            }`}
          >
            <div className="flex items-center gap-2">
              <Wrench className="w-4 h-4" />
              Maintenance
            </div>
          </button>
          <button
            onClick={() => setActiveTab('settings')}
            className={`px-4 py-2 font-medium transition-colors ${
              activeTab === 'settings'
                ? 'text-red-600 border-b-2 border-red-600'
                : 'text-gray-600 hover:text-gray-900'
            }`}
          >
            <div className="flex items-center gap-2">
              <Settings className="w-4 h-4" />
              Settings
            </div>
          </button>
        </div>
      </nav>

      <main className="flex-1 overflow-y-auto">
        {activeTab === 'scanner' && (
          <div className="h-full flex">
            <div className="w-1/3 border-r border-gray-200 overflow-y-auto bg-white">
              {groups.length === 0 ? (
                <div className="p-8 text-center">
                  <Upload className="w-16 h-16 text-gray-300 mx-auto mb-4" />
                  <p className="text-gray-500">No files scanned yet</p>
                  <p className="text-sm text-gray-400 mt-1">Click "Scan Library" to get started</p>
                </div>
              ) : (
                <div>
                  <div className="p-4 bg-gray-50 border-b border-gray-200">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-4 text-sm">
                        <span className="font-medium text-gray-700">{groups.length} groups</span>
                        <span className="text-gray-500">
                          {groups.reduce((sum, g) => sum + g.total_changes, 0)} changes
                        </span>
                        {getSuccessCount() > 0 && (
                          <span className="text-green-600">✓ {getSuccessCount()} written</span>
                        )}
                        {getFailedCount() > 0 && (
                          <span className="text-red-600">✗ {getFailedCount()} failed</span>
                        )}
                      </div>
                      <div className="flex gap-2">
                        <button
                          onClick={() => {
                            const allGroupIds = groups.flatMap(g => g.files.map(f => f.id));
                            setSelectedFiles(new Set(allGroupIds));
                          }}
                          className="px-3 py-1.5 text-xs bg-white border border-gray-300 hover:bg-gray-50 text-gray-700 rounded transition-colors"
                        >
                          Select All
                        </button>
                        <button
                          onClick={() => setSelectedFiles(new Set())}
                          className="px-3 py-1.5 text-xs bg-white border border-gray-300 hover:bg-gray-50 text-gray-700 rounded transition-colors"
                        >
                          Deselect All
                        </button>
                      </div>
                    </div>
                  </div>
                  {groups.map((group) => (
                    <div key={group.id} className="border-b border-gray-100">
                      <div
                        className={`p-4 hover:bg-gray-50 cursor-pointer ${
                          selectedGroup?.id === group.id ? 'bg-red-50' : ''
                        }`}
                        onClick={() => setSelectedGroup(group)}
                      >
                        <div className="flex items-center gap-2">
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              toggleGroup(group.id);
                            }}
                          >
                            {expandedGroups.has(group.id) ? (
                              <ChevronDown className="w-4 h-4 text-gray-400" />
                            ) : (
                              <ChevronRight className="w-4 h-4 text-gray-400" />
                            )}
                          </button>
                          {getGroupIcon(group.group_type)}
                          <input
                            type="checkbox"
                            checked={group.files.every(f => selectedFiles.has(f.id))}
                            onChange={(e) => {
                              e.stopPropagation();
                              selectAllInGroup(group, e.target.checked);
                            }}
                            className="rounded"
                          />
                          <div className="flex-1 min-w-0">
                            <div className="font-medium text-gray-900 text-sm truncate">
                              {group.group_name}
                            </div>
                            <div className="text-xs text-gray-500">
                              {group.files.length} files • {group.group_type}
                              {group.total_changes > 0 && (
                                <span className="ml-2 px-2 py-0.5 bg-yellow-100 text-yellow-800 rounded">
                                  {group.total_changes} changes
                                </span>
                              )}
                            </div>
                          </div>
                        </div>
                      </div>
                      
                      {expandedGroups.has(group.id) && (
                        <div className="bg-gray-50">
                          {group.files.map((file) => (
                            <div
                              key={file.id}
                              className="pl-10 pr-4 py-3 border-t border-gray-200 hover:bg-gray-100"
                            >
                              <div className="flex items-start gap-3">
                                <input
                                  type="checkbox"
                                  checked={selectedFiles.has(file.id)}
                                  onChange={(e) => {
                                    e.stopPropagation();
                                    const newSet = new Set(selectedFiles);
                                    if (newSet.has(file.id)) {
                                      newSet.delete(file.id);
                                    } else {
                                      newSet.add(file.id);
                                    }
                                    setSelectedFiles(newSet);
                                  }}
                                  className="mt-1 rounded"
                                />
                                {getFileStatusIcon(file.id)}
                                <div className="flex-1 min-w-0">
                                  <div className="text-sm text-gray-900 truncate">
                                    {file.filename}
                                  </div>
                                  {Object.keys(file.changes).length > 0 && (
                                    <div className="text-xs text-yellow-600 mt-1">
                                      {Object.keys(file.changes).length} changes
                                    </div>
                                  )}
                                </div>
                              </div>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="flex-1 overflow-y-auto p-6 bg-gray-50">
              {!selectedGroup ? (
                <div className="h-full flex items-center justify-center">
                  <div className="text-center">
                    <FileAudio className="w-16 h-16 text-gray-300 mx-auto mb-4" />
                    <p className="text-gray-500">Select a group to view details</p>
                  </div>
                </div>
              ) : (
                <div className="max-w-3xl mx-auto">
                  <MetadataDisplay metadata={selectedGroup.metadata} groupInfo={selectedGroup} />
                </div>
              )}
            </div>
          </div>
        )}

        {activeTab === 'maintenance' && (
          <MaintenanceTab />
        )}

        {activeTab === 'settings' && (
          <SettingsTab config={config} setConfig={setConfig} saveConfig={saveConfig} testConnection={testConnection} />
        )}
      </main>

      {showTagInspector && (
        <RawTagInspector 
          isOpen={showTagInspector} 
          onClose={() => setShowTagInspector(false)} 
        />
      )}
    </div>
  );
}

function MetadataDisplay({ metadata, groupInfo }) {
  const meta = metadata || {};
  
  return (
    <div className="bg-white rounded-xl shadow-sm p-8 space-y-8">
      <div className="space-y-2">
        <h1 className="text-3xl font-bold text-gray-900 leading-tight">
          {meta.title || 'Untitled'}
        </h1>
        {meta.subtitle && (
          <p className="text-lg text-gray-600">{meta.subtitle}</p>
        )}
      </div>

      <div className="flex items-center gap-6 text-sm pb-6 border-b border-gray-100">
        <div>
          <span className="text-gray-500">by </span>
          <span className="font-medium text-gray-900">{meta.author || 'Unknown Author'}</span>
        </div>
        {meta.year && (
          <div className="text-gray-500">
            {meta.year}
          </div>
        )}
        {groupInfo && (
          <div className="text-gray-500">
            {groupInfo.files.length} files
          </div>
        )}
      </div>

      {meta.series && (
        <div className="space-y-2">
          <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide">
            Series
          </div>
          <div className="inline-flex items-center gap-2 px-4 py-2 bg-gray-50 rounded-lg border border-gray-200">
            <Book className="w-4 h-4 text-gray-600" />
            <span className="font-medium text-gray-900">{meta.series}</span>
            {meta.sequence && (
              <span className="text-gray-600">#{meta.sequence}</span>
            )}
          </div>
        </div>
      )}

      {meta.narrator && (
        <div className="space-y-2">
          <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide">
            Narrated by
          </div>
          <p className="text-gray-900">{meta.narrator}</p>
        </div>
      )}

      {meta.genres && meta.genres.length > 0 && (
        <div className="space-y-3">
          <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide">
            Genres
          </div>
          <div className="flex flex-wrap gap-2">
            {meta.genres.map((genre, idx) => (
              <span 
                key={idx}
                className="inline-flex items-center px-3 py-1.5 bg-gray-900 text-white text-sm font-medium rounded-full"
              >
                {genre}
              </span>
            ))}
          </div>
        </div>
      )}

      {meta.description && (
        <div className="space-y-3">
          <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide">
            About
          </div>
          <p className="text-gray-700 leading-relaxed text-sm">
            {meta.description}
          </p>
        </div>
      )}

      {(meta.publisher || meta.isbn) && (
        <div className="pt-6 border-t border-gray-100">
          <div className="grid grid-cols-2 gap-6 text-sm">
            {meta.publisher && (
              <div>
                <div className="text-xs text-gray-500 mb-1">Publisher</div>
                <div className="text-gray-900">{meta.publisher}</div>
              </div>
            )}
            {meta.isbn && (
              <div>
                <div className="text-xs text-gray-500 mb-1">ISBN</div>
                <div className="text-gray-900 font-mono text-xs">{meta.isbn}</div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function MaintenanceTab() {
  return (
    <div className="p-6 overflow-y-auto">
      <div className="max-w-4xl mx-auto">
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h2 className="text-xl font-semibold mb-4">Library Maintenance</h2>
          <p className="text-gray-600 mb-6">
            Advanced maintenance features for AudiobookShelf and local library management.
          </p>
          
          <div className="space-y-6">
            <div className="border rounded-lg p-4">
              <h3 className="text-lg font-semibold mb-3">AudiobookShelf Server</h3>
              <div className="grid gap-3">
                <button 
                  onClick={async () => {
                    if (confirm('Restart AudiobookShelf Docker container?')) {
                      try {
                        await invoke('restart_abs_docker');
                        alert('AudiobookShelf container restarted successfully!');
                      } catch (error) {
                        alert('Failed to restart container: ' + error);
                      }
                    }
                  }}
                  className="btn btn-primary"
                >
                  🔄 Restart Docker Container
                </button>
                
                <button 
                  onClick={async () => {
                    if (confirm('Force a library rescan in AudiobookShelf? This may take a while.')) {
                      try {
                        await invoke('force_abs_rescan');
                        alert('Library rescan triggered successfully!');
                      } catch (error) {
                        alert('Failed to trigger rescan: ' + error);
                      }
                    }
                  }}
                  className="btn btn-primary"
                >
                  📚 Force Library Rescan
                </button>
                
                <button 
                  onClick={async () => {
                    if (confirm('Clear all AudiobookShelf caches?')) {
                      try {
                        await invoke('clear_abs_cache');
                        alert('AudiobookShelf caches cleared successfully!');
                      } catch (error) {
                        alert('Failed to clear caches: ' + error);
                      }
                    }
                  }}
                  className="btn btn-secondary"
                >
                  🧹 Clear Server Cache
                </button>
              </div>
            </div>

            <div className="border rounded-lg p-4">
              <h3 className="text-lg font-semibold mb-3">Genre Management</h3>
              <div className="grid gap-3">
                <button 
                  onClick={async () => {
                    if (confirm('Clear unused genres from the dropdown menu? This only removes genres not used by any books.')) {
                      try {
                        const result = await invoke('clear_all_genres');
                        alert(result);
                      } catch (error) {
                        alert('Failed to clear genres: ' + error);
                      }
                    }
                  }}
                  className="btn btn-secondary"
                >
                  🧹 Clear Unused Genres from Dropdown
                </button>
                
                <button 
                  onClick={async () => {
                    if (confirm('Normalize all genres to approved list?')) {
                      try {
                        const result = await invoke('normalize_genres');
                        alert(result);
                      } catch (error) {
                        alert('Failed to normalize genres: ' + error);
                      }
                    }
                  }}
                  className="btn btn-secondary"
                >
                  📋 Normalize Book Genres
                </button>
              </div>
            </div>

            <div className="border rounded-lg p-4">
              <h3 className="text-lg font-semibold mb-3">Local Library</h3>
              <div className="grid gap-3">
                <button 
                  onClick={async () => {
                    if (confirm('Clear all cached metadata? This will force fresh lookups on next scan.')) {
                      try {
                        await invoke('clear_cache');
                        alert('Cache cleared successfully!');
                      } catch (error) {
                        alert('Failed to clear cache: ' + error);
                      }
                    }
                  }}
                  className="btn btn-secondary"
                >
                  💾 Clear Metadata Cache
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function SettingsTab({ config, setConfig, saveConfig, testConnection }) {
  return (
    <div className="p-6 overflow-y-auto">
      <div className="max-w-4xl mx-auto space-y-6">
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h3 className="text-lg font-semibold mb-4">Audiobookshelf Connection</h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Base URL</label>
              <input
                type="text"
                value={config.abs_base_url}
                onChange={(e) => setConfig({ ...config, abs_base_url: e.target.value })}
                placeholder="http://localhost:13378"
                className="input"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">API Token</label>
              <input
                type="password"
                value={config.abs_api_token}
                onChange={(e) => setConfig({ ...config, abs_api_token: e.target.value })}
                placeholder="Enter API token"
                className="input"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Library ID</label>
              <input
                type="text"
                value={config.abs_library_id}
                onChange={(e) => setConfig({ ...config, abs_library_id: e.target.value })}
                placeholder="lib_xxxxxxxxxxxxx"
                className="input"
              />
            </div>
            <div className="flex gap-2">
              <button onClick={testConnection} className="btn btn-secondary">Test Connection</button>
              <button onClick={() => saveConfig(config)} className="btn btn-primary">Save Settings</button>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h3 className="text-lg font-semibold mb-4">API Keys</h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">OpenAI API Key</label>
              <input
                type="password"
                value={config.openai_api_key || ''}
                onChange={(e) => setConfig({ ...config, openai_api_key: e.target.value })}
                placeholder="sk-..."
                className="input"
              />
              <p className="text-xs text-gray-500 mt-1">
                For AI-powered metadata extraction and narrator detection
              </p>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Google Books API Key</label>
              <input
                type="password"
                value={config.google_books_api_key || ''}
                onChange={(e) => setConfig({ ...config, google_books_api_key: e.target.value })}
                placeholder="AIza..."
                className="input"
              />
              <p className="text-xs text-gray-500 mt-1">
                For metadata enrichment (optional, for high volume use)
              </p>
            </div>
            <button onClick={() => saveConfig(config)} className="btn btn-primary">Save Settings</button>
          </div>
        </div>

        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h3 className="text-lg font-semibold mb-4">Audible Integration</h3>
          <div className="space-y-4">
            <div>
              <label className="flex items-center gap-2 mb-3">
                <input 
                  type="checkbox" 
                  checked={config.audible_enabled || false} 
                  onChange={(e) => setConfig({ ...config, audible_enabled: e.target.checked })}
                  className="rounded" 
                />
                <span className="text-sm font-medium text-gray-700">Enable Audible (Primary Source)</span>
              </label>
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Audible CLI Path</label>
              <input
                type="text"
                value={config.audible_cli_path || 'audible'}
                onChange={(e) => setConfig({ ...config, audible_cli_path: e.target.value })}
                placeholder="audible"
                className="input"
              />
              <p className="text-xs text-gray-500 mt-1">
                ✅ Authenticated via: audible quickstart
              </p>
            </div>
            
            <button onClick={() => saveConfig(config)} className="btn btn-primary">Save Settings</button>
          </div>
        </div>

        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h3 className="text-lg font-semibold mb-4">Processing Options</h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Parallel Workers
              </label>
              <input
                type="number"
                min="1"
                max="50"
                value={config.max_workers || 10}
                onChange={(e) => setConfig({ ...config, max_workers: parseInt(e.target.value) })}
                className="input"
              />
              <p className="text-xs text-gray-500 mt-1">
                M4 Mac recommended: 20-30 workers (default: 10)
              </p>
            </div>
            
            <div>
              <label className="flex items-center gap-2">
                <input 
                  type="checkbox" 
                  checked={config.skip_unchanged || false} 
                  onChange={(e) => setConfig({ ...config, skip_unchanged: e.target.checked })}
                  className="rounded" 
                />
                <span className="text-sm font-medium text-gray-700">Skip Unchanged Files</span>
              </label>
              <p className="text-xs text-gray-500 mt-1">
                Only process files with missing or incorrect metadata
              </p>
            </div>

            <div>
              <label className="flex items-center gap-2">
                <input 
                  type="checkbox" 
                  checked={config.backup_tags} 
                  onChange={(e) => setConfig({ ...config, backup_tags: e.target.checked })}
                  className="rounded" 
                />
                <span className="text-sm text-gray-700">Backup original tags</span>
              </label>
            </div>

            <div>
              <label className="flex items-center gap-2">
                <input 
                  type="checkbox" 
                  checked={config.genre_enforcement} 
                  onChange={(e) => setConfig({ ...config, genre_enforcement: e.target.checked })}
                  className="rounded" 
                />
                <span className="text-sm text-gray-700">Enforce approved genres</span>
              </label>
            </div>

            <button onClick={() => saveConfig(config)} className="btn btn-primary">Save Settings</button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
