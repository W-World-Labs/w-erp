import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned file display", () => {
  it("renders file and attachment display metadata from backend labels", () => {
    const apiClientSource = readFileSync(
      resolve(root, "integrations/api/client.ts"),
      "utf8",
    );
    const fileManagerSource = readFileSync(
      resolve(root, "components/common/FileManager.tsx"),
      "utf8",
    );
    const fileUploaderSource = readFileSync(
      resolve(root, "components/common/FileUploader.tsx"),
      "utf8",
    );
    const attachmentManagerSource = readFileSync(
      resolve(root, "components/common/AttachmentManager.tsx"),
      "utf8",
    );
    const employeeDocumentsSource = readFileSync(
      resolve(root, "components/employees/EmployeeDocuments.tsx"),
      "utf8",
    );
    const projectDocumentsSource = readFileSync(
      resolve(root, "components/projects/ProjectDocuments.tsx"),
      "utf8",
    );

    expect(apiClientSource).toContain("size_label?: string");
    expect(apiClientSource).toContain("created_at_label?: string");
    expect(apiClientSource).toContain("metadata_label?: string");

    expect(fileManagerSource).toContain("file.fileSizeLabel");
    expect(fileManagerSource).toContain("file.uploadDateLabel");
    expect(fileManagerSource).not.toContain("formatFileSize");
    expect(fileManagerSource).not.toContain("formatDate");
    expect(fileManagerSource).not.toContain("toFixed");
    expect(fileManagerSource).not.toContain("toLocaleDateString");
    expect(fileManagerSource).not.toContain("new Date(");

    expect(fileUploaderSource).not.toContain("formatFileSize");
    expect(fileUploaderSource).not.toContain("toFixed");

    expect(attachmentManagerSource).toContain("att.upload_date_label");
    expect(attachmentManagerSource).toContain("files.list(entityType, entityId)");
    expect(attachmentManagerSource).toContain("att.file_icon");
    expect(attachmentManagerSource).toContain("att.file_icon_class_name");
    expect(attachmentManagerSource).toContain("att.delete_action");
    expect(attachmentManagerSource).not.toContain("new Date(att.upload_date)");
    expect(attachmentManagerSource).not.toContain("toLocaleDateString");
    expect(attachmentManagerSource).not.toContain("api.get<{ data: any[] }>");
    expect(attachmentManagerSource).not.toContain("tableNames");
    expect(attachmentManagerSource).not.toContain("idFields");
    expect(attachmentManagerSource).not.toContain("getFileIcon");
    expect(attachmentManagerSource).not.toContain("fileType.startsWith");

    expect(employeeDocumentsSource).toContain("doc.upload_date_label");
    expect(employeeDocumentsSource).toContain("/api/employee_documents/overview");
    expect(employeeDocumentsSource).toContain("documentsOverview?.documents");
    expect(employeeDocumentsSource).toContain("documentsOverview?.document_type_options");
    expect(employeeDocumentsSource).toContain("documentsOverview?.upload_action");
    expect(employeeDocumentsSource).toContain("doc.document_type_label");
    expect(employeeDocumentsSource).toContain("doc.file_icon");
    expect(employeeDocumentsSource).toContain("doc.delete_action");
    expect(employeeDocumentsSource).not.toContain("new Date(doc.upload_date)");
    expect(employeeDocumentsSource).not.toContain("toLocaleDateString");
    expect(employeeDocumentsSource).not.toContain("api.get<{ data: any[]; count: number }>");
    expect(employeeDocumentsSource).not.toContain("getFileIcon");
    expect(employeeDocumentsSource).not.toContain("getDocumentTypeLabel");
    expect(employeeDocumentsSource).not.toContain("fileName.split");

    expect(projectDocumentsSource).toContain("doc.upload_date_label");
    expect(projectDocumentsSource).toContain("/api/project_documents/overview");
    expect(projectDocumentsSource).toContain("documentsOverview?.documents");
    expect(projectDocumentsSource).toContain("documentsOverview?.document_type_options");
    expect(projectDocumentsSource).toContain("doc.document_type_label");
    expect(projectDocumentsSource).toContain("doc.file_icon");
    expect(projectDocumentsSource).toContain("doc.delete_action");
    expect(projectDocumentsSource).not.toContain("new Date(doc.upload_date)");
    expect(projectDocumentsSource).not.toContain("toLocaleDateString");
    expect(projectDocumentsSource).not.toContain("api.get<{ data: any[]; count: number }>");
    expect(projectDocumentsSource).not.toContain("getFileIcon");
    expect(projectDocumentsSource).not.toContain("getDocumentTypeLabel");
    expect(projectDocumentsSource).not.toContain("fileName.split");
  });

  it("delegates upload storage naming to backend file API", () => {
    const uploadHookSource = readFileSync(
      resolve(root, "hooks/useFileUpload.ts"),
      "utf8",
    );

    expect(uploadHookSource).toContain("files.upload(file)");
    expect(uploadHookSource).toContain("uploadResult.id");
    expect(uploadHookSource).toContain("uploadResult.original_name");
    expect(uploadHookSource).not.toContain("Date.now()");
    expect(uploadHookSource).not.toContain("crypto.randomUUID");
    expect(uploadHookSource).not.toContain("generatePath");
    expect(uploadHookSource).not.toContain('formData.append("path"');
    expect(uploadHookSource).not.toContain("api.post('/api/files/upload'");
  });

  it("delegates attachment and document file storage to backend file ids", () => {
    const attachmentManagerSource = readFileSync(
      resolve(root, "components/common/AttachmentManager.tsx"),
      "utf8",
    );
    const employeeDocumentsSource = readFileSync(
      resolve(root, "components/employees/EmployeeDocuments.tsx"),
      "utf8",
    );
    const projectDocumentsSource = readFileSync(
      resolve(root, "components/projects/ProjectDocuments.tsx"),
      "utf8",
    );
    const combined = `${attachmentManagerSource}\n${employeeDocumentsSource}\n${projectDocumentsSource}`;

    expect(combined).toContain("files.uploadForEntity");
    expect(combined).toContain("uploadResult.id");
    expect(combined).toContain("uploadResult.original_name");
    expect(combined).toContain("files.download");
    expect(combined).toContain("api.runAction(action");
    expect(combined).not.toContain("api.delete(action.endpoint)");

    expect(combined).not.toContain("Date.now()");
    expect(combined).not.toContain("fileExt");
    expect(combined).not.toContain("formData.append('path'");
    expect(combined).not.toContain('formData.append("path"');
    expect(combined).not.toContain("/api/files/download");
    expect(combined).not.toContain("TODO: implement file download");
    expect(combined).not.toContain("user.id");
    expect(combined).not.toContain("dbError");
    expect(combined).not.toContain("storageError");
    expect(combined).not.toContain("files.delete(attachment.file_path)");
    expect(combined).not.toContain("api.delete(`/api/employee_documents/${document.id}`)");
    expect(combined).not.toContain("api.delete(`/api/project_documents/${document.id}`)");
    expect(combined).not.toContain("api.post('/api/employee_documents'");
    expect(combined).not.toContain("api.post('/api/project_documents'");
    expect(combined).not.toContain("确定要删除\"${document.file_name}\"吗？");
    expect(combined).not.toContain("确定要删除\"${attachment.file_name}\"吗？");
  });
});
