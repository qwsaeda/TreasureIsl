/* ea:   0x00061210 */
/* name: sub_61210 */
/* size: 104 */
DWORD __fastcall sub_61210(HANDLE *a1, void *a2, DWORD a3)
{
  bool v6; // zf
  DWORD result; // r0
  DWORD NumberOfBytesRead; // [sp+4h] [bp-14h] BYREF

  if ( (*((unsigned __int8 (__fastcall **)(HANDLE *))*a1 + 9))(a1) == 0 || a2 == nullptr )
    __debugbreak();
  v6 = !ReadFile(a1[1], a2, a3, &NumberOfBytesRead, nullptr);
  result = -1;
  if ( !v6 )
    return NumberOfBytesRead;
  return result;
}

