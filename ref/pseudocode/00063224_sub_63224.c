/* ea:   0x00063224 */
/* name: sub_63224 */
/* size: 212 */
int __fastcall sub_63224(_DWORD *a1, const WCHAR *a2, int a3)
{
  DWORD dwCreationDisposition; // r0
  DWORD v7; // r1
  HANDLE FileW; // r0
  int v9; // r1
  DWORD dwDesiredAccess; // [sp+Ch] [bp-14h]

  if ( a2 == nullptr )
    __debugbreak();
  if ( (*(unsigned __int8 (__fastcall **)(_DWORD *))(*a1 + 36))(a1) != 0 )
    (*(void (__fastcall **)(_DWORD *))(*a1 + 8))(a1);
  if ( a3 != 0 )
  {
    if ( a3 == 1 )
    {
      v7 = 0x40000000;
      dwCreationDisposition = 2;
    }
    else
    {
      dwCreationDisposition = dwDesiredAccess;
      if ( a3 == 2 )
        v7 = -1073741824;
      else
        v7 = dwDesiredAccess;
    }
  }
  else
  {
    v7 = 0x80000000;
    dwCreationDisposition = 3;
  }
  FileW = CreateFileW(a2, v7, 3u, nullptr, dwCreationDisposition, 0x80u, nullptr);
  v9 = *a1;
  a1[1] = FileW;
  return (*(int (__fastcall **)(_DWORD *))(v9 + 36))(a1);
}

