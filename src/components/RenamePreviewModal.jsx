// RenamePreviewModal.jsx - Add this component to your app

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { FileText, Folder, AlertCircle, CheckCircle, ArrowRight } from 'lucide-react';

export function RenamePreviewModal({ 
  selectedFiles, 
  metadata, 
  onConfirm, 
  onCancel, 
  config 
}) {
  const [previews, setPreviews] = useState([]);
  const [reorganize, setReorganize] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    generatePreviews();
  }, [selectedFiles, metadata, reorganize]);

  const generatePreviews = async () => {
    setLoading(true);
    const previewResults = [];

    for (const filePath of selectedFiles) {
      try {
        const newPath = await invoke('preview_rename', {
          filePath,
          metadata: {
            title: metadata.title || '',
            author: metadata.author || '',
            series: metadata.series || null,
            sequence: metadata.sequence || null,
            year: metadata.year || null,
          },
          reorganize,
          libraryRoot: reorganize ? config.library_root : null,
        });

        previewResults.push({
          oldPath: filePath,
          newPath,
          changed: filePath !== newPath,
        });
      } catch (error) {
        console.error('Preview error:', error);
        previewResults.push({
          oldPath: filePath,
          newPath: filePath,
          changed: false,
          error: error.toString(),
        });
      }
    }

    setPreviews(previewResults);
    setLoading(false);
  };

  const handleConfirm = () => {
    onConfirm(reorganize);
  };

  const anyChanges = previews.some(p => p.changed);
  const anyErrors = previews.some(p => p.error);

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-xl shadow-2xl max-w-4xl w-full max-h-[80vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="p-6 border-b border-gray-200">
          <h2 className="text-2xl font-bold text-gray-900 mb-2">
            Rename Files Preview
          </h2>
          <p className="text-gray-600">
            Review the proposed file name changes before applying them
          </p>
        </div>

        {/* Options */}
        <div className="px-6 py-4 bg-gray-50 border-b border-gray-200">
          <label className="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={reorganize}
              onChange={(e) => setReorganize(e.target.checked)}
              className="w-5 h-5 rounded border-gray-300 text-red-600 focus:ring-red-500"
            />
            <div>
              <div className="font-medium text-gray-900">
                Reorganize into Author/Series Folders
              </div>
              <div className="text-sm text-gray-600">
                Move files into structured folders: Author/Series/Book
              </div>
            </div>
          </label>
        </div>

        {/* Preview List */}
        <div className="flex-1 overflow-y-auto p-6">
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-red-600"></div>
              <span className="ml-3 text-gray-600">Generating preview...</span>
            </div>
          ) : (
            <div className="space-y-4">
              {previews.map((preview, idx) => (
                <div
                  key={idx}
                  className={`p-4 rounded-lg border-2 ${
                    preview.error
                      ? 'bg-red-50 border-red-200'
                      : preview.changed
                      ? 'bg-blue-50 border-blue-200'
                      : 'bg-gray-50 border-gray-200'
                  }`}
                >
                  {preview.error ? (
                    <div className="flex items-start gap-3">
                      <AlertCircle className="w-5 h-5 text-red-600 mt-0.5 flex-shrink-0" />
                      <div className="flex-1 min-w-0">
                        <div className="font-medium text-red-900 mb-1">Error</div>
                        <div className="text-sm text-red-700">{preview.error}</div>
                      </div>
                    </div>
                  ) : preview.changed ? (
                    <div className="space-y-3">
                      <div className="flex items-start gap-3">
                        <FileText className="w-5 h-5 text-gray-600 mt-1 flex-shrink-0" />
                        <div className="flex-1 min-w-0">
                          <div className="text-xs font-medium text-gray-500 mb-1">
                            Original
                          </div>
                          <div className="text-sm text-gray-900 font-mono bg-white px-3 py-2 rounded border border-gray-200 break-all">
                            {preview.oldPath.split('/').pop()}
                          </div>
                          {reorganize && (
                            <div className="text-xs text-gray-500 mt-1">
                              📁 {preview.oldPath.split('/').slice(0, -1).join('/')}
                            </div>
                          )}
                        </div>
                      </div>

                      <div className="flex justify-center">
                        <ArrowRight className="w-5 h-5 text-blue-600" />
                      </div>

                      <div className="flex items-start gap-3">
                        <CheckCircle className="w-5 h-5 text-green-600 mt-1 flex-shrink-0" />
                        <div className="flex-1 min-w-0">
                          <div className="text-xs font-medium text-gray-500 mb-1">
                            New Name
                          </div>
                          <div className="text-sm text-gray-900 font-mono bg-green-50 px-3 py-2 rounded border border-green-200 break-all">
                            {preview.newPath.split('/').pop()}
                          </div>
                          {reorganize && (
                            <div className="text-xs text-green-700 mt-1 flex items-center gap-1">
                              <Folder className="w-3 h-3" />
                              {preview.newPath.split('/').slice(0, -1).join('/')}
                            </div>
                          )}
                        </div>
                      </div>
                    </div>
                  ) : (
                    <div className="flex items-center gap-3">
                      <CheckCircle className="w-5 h-5 text-gray-400" />
                      <div className="text-sm text-gray-600">
                        No changes needed for this file
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}

          {!loading && !anyChanges && !anyErrors && (
            <div className="text-center py-12">
              <CheckCircle className="w-16 h-16 text-gray-300 mx-auto mb-4" />
              <p className="text-gray-500">
                File names already match the metadata. No changes needed.
              </p>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="p-6 border-t border-gray-200 bg-gray-50">
          <div className="flex items-center justify-between">
            <div className="text-sm text-gray-600">
              {anyChanges && (
                <span>
                  {previews.filter(p => p.changed).length} file(s) will be renamed
                </span>
              )}
            </div>
            <div className="flex gap-3">
              <button
                onClick={onCancel}
                className="px-4 py-2 text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleConfirm}
                disabled={!anyChanges || anyErrors || loading}
                className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors flex items-center gap-2"
              >
                <CheckCircle className="w-4 h-4" />
                Confirm & Rename
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// Usage in App.jsx:

// Add state for rename modal
const [showRenameModal, setShowRenameModal] = useState(false);
const [renameMetadata, setRenameMetadata] = useState(null);

// Modify handleWrite to show rename preview
const handleWrite = async () => {
  if (selectedFiles.size === 0) {
    alert('No files selected');
    return;
  }
  
  // Get metadata from selected group
  const group = groups.find(g => g.files.some(f => selectedFiles.has(f.id)));
  if (!group) return;
  
  setRenameMetadata(group.metadata);
  setShowRenameModal(true);
};

const handleRenameConfirm = async (reorganize) => {
  setShowRenameModal(false);
  setWriting(true);
  
  try {
    const filesMap = {};
    groups.forEach(group => {
      group.files.forEach(file => {
        filesMap[file.id] = {
          path: file.path,
          changes: file.changes
        };
      });
    });
    
    const result = await invoke('write_and_rename', {
      request: {
        file_ids: Array.from(selectedFiles),
        files: filesMap,
        backup: config.backup_tags
      },
      config,
      renameFilesFlag: true,
      reorganize
    });
    
    setWriting(false);
    
    const { write_result, rename_results } = result;
    const renamed = rename_results.filter(r => r.success).length;
    
    alert(
      `✅ Tags written: ${write_result.success} files\n` +
      `✅ Files renamed: ${renamed} files\n` +
      (write_result.failed > 0 ? `❌ Failed: ${write_result.failed} files\n` : '')
    );
    
    setSelectedFiles(new Set());
  } catch (error) {
    console.error('Write failed:', error);
    setWriting(false);
    alert('Write failed: ' + error);
  }
};

// In your JSX, add the modal:
{showRenameModal && (
  <RenamePreviewModal
    selectedFiles={Array.from(selectedFiles).map(id => {
      for (const group of groups) {
        const file = group.files.find(f => f.id === id);
        if (file) return file.path;
      }
      return null;
    }).filter(Boolean)}
    metadata={renameMetadata}
    config={config}
    onConfirm={handleRenameConfirm}
    onCancel={() => setShowRenameModal(false)}
  />
)}