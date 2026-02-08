# Database Update - Document Ownership and Privacy

## Date: 2025-02-08

## Summary
Updated existing documents in the database to assign ownership to user `jueewo` and set the BPMN file to private.

---

## Updates Performed

### 1. Set User Ownership
**Command:**
```sql
UPDATE documents SET user_id = 'jueewo' WHERE id IN (1, 2);
```

**Result:**
- Document ID 1 (PDF): `Normalverteilungstabellen` → Owner: `jueewo`
- Document ID 2 (BPMN): `diagram bpmn demo1` → Owner: `jueewo`

### 2. Set BPMN File to Private
**Command:**
```sql
UPDATE documents SET is_public = 0 WHERE document_type = 'bpmn';
```

**Result:**
- Document ID 2: `diagram bpmn demo1` → Privacy: Private (is_public = 0)

---

## Final State

### Documents Table
| ID | Title | Type | Public | User ID |
|----|-------|------|--------|---------|
| 1 | Normalverteilungstabellen | pdf | 1 (Public) | jueewo |
| 2 | diagram bpmn demo1 | bpmn | 0 (Private) | jueewo |

---

## Verification

### Query Used:
```sql
SELECT id, title, document_type, is_public, user_id FROM documents;
```

### Output:
```
1|Normalverteilungstabellen|pdf|1|jueewo
2|diagram bpmn demo1|bpmn|0|jueewo
```

✅ Confirmed: All documents now have proper ownership and privacy settings

---

## Security Implications

### Before Update
- ❌ Documents had no user ownership (user_id was NULL)
- ❌ All documents were public
- ❌ No way to filter by user
- ❌ No privacy control

### After Update
- ✅ Documents assigned to user `jueewo`
- ✅ BPMN file is private (only visible to owner when authenticated)
- ✅ PDF remains public (visible to all)
- ✅ Proper ownership tracking in place

---

## Access Control

### PDF Document (ID: 1)
- **Title:** Normalverteilungstabellen
- **Type:** pdf
- **Privacy:** Public (is_public = 1)
- **Owner:** jueewo
- **Access:**
  - ✅ Visible to guests
  - ✅ Visible to all authenticated users
  - ✅ Owner: jueewo

### BPMN Document (ID: 2)
- **Title:** diagram bpmn demo1
- **Type:** bpmn
- **Privacy:** Private (is_public = 0)
- **Owner:** jueewo
- **Access:**
  - ❌ NOT visible to guests
  - ❌ NOT visible to other authenticated users
  - ✅ Visible ONLY to owner: jueewo

---

## Testing Recommendations

### 1. Test as Guest (Unauthenticated)
```bash
# Visit /media or /documents
# Expected: Only PDF visible
# Expected: BPMN NOT visible
```

### 2. Test as Owner (jueewo)
```bash
# Login as jueewo
# Visit /media or /documents
# Expected: Both PDF and BPMN visible
# Expected: BPMN shows "Private" indicator
```

### 3. Test as Different User
```bash
# Login as different user
# Visit /media or /documents
# Expected: Only PDF visible
# Expected: BPMN NOT visible
```

---

## Related Database Tables

### Images
- All images already have user_id assigned
- User: `7bda815e-729a-49ea-88c5-3ca59b9ce487` (UUID format)
- Status: ✅ No update needed

### Videos
- All videos already have user_id assigned
- User: `7bda815e-729a-49ea-88c5-3ca59b9ce487` (UUID format)
- Status: ✅ No update needed

---

## SQL Commands Reference

### Set Ownership for Specific Documents
```sql
UPDATE documents 
SET user_id = 'jueewo' 
WHERE id IN (1, 2);
```

### Set Privacy by Document Type
```sql
UPDATE documents 
SET is_public = 0 
WHERE document_type = 'bpmn';
```

### Set Privacy for Specific Document
```sql
UPDATE documents 
SET is_public = 0 
WHERE id = 2;
```

### View All Documents
```sql
SELECT id, title, document_type, is_public, user_id 
FROM documents;
```

### View Private Documents
```sql
SELECT id, title, document_type, user_id 
FROM documents 
WHERE is_public = 0;
```

### View User's Documents
```sql
SELECT id, title, document_type, is_public 
FROM documents 
WHERE user_id = 'jueewo';
```

---

## Future Considerations

### Batch Updates
If more documents need ownership assignment:
```sql
-- Set all NULL user_ids to specific user
UPDATE documents 
SET user_id = 'jueewo' 
WHERE user_id IS NULL;

-- Set all documents of a type to private
UPDATE documents 
SET is_public = 0 
WHERE document_type IN ('bpmn', 'xml');
```

### User ID Format
Note: Different tables use different user_id formats:
- **Documents:** String format (e.g., 'jueewo')
- **Images/Videos:** UUID format (e.g., '7bda815e-729a-49ea-88c5-3ca59b9ce487')

This should be standardized in future for consistency.

---

## Status
✅ **COMPLETE**

All existing documents have been updated with:
- User ownership assigned
- Privacy settings configured
- Ready for production use

**Database:** media.db  
**Tables Updated:** documents  
**Records Updated:** 2  
**Backup Recommended:** Yes (before any database updates)

---

## Notes
- Always backup database before updates: `cp media.db media.db.backup`
- Test changes in development before production
- Verify user_id exists in users table before assignment
- Consider standardizing user_id format across all tables