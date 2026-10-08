/* ea:   0x000644F8 */
/* name: sub_644F8 */
/* size: 188 */
int __fastcall sub_644F8(int a1, const WCHAR *a2)
{
  int v3; // r4
  DWORD v4; // r4
  HANDLE v6[5]; // [sp+0h] [bp-28h] BYREF
  int v7; // [sp+14h] [bp-14h]

  if ( a2 == nullptr )
    __debugbreak();
  v3 = 0;
  v6[1] = (HANDLE)-1;
  v7 = 0;
  v6[0] = off_6D220;
  v6[2] = nullptr;
  if ( (unsigned __int8)sub_61F98((int)v6, a2, 0, 0) != 0 )
  {
    v4 = sub_1E67C(v6);
    if ( (*((unsigned __int8 (__fastcall **)(HANDLE *))v6[0] + 9))(v6) == 0 )
      __debugbreak();
    v3 = (*(int (__fastcall **)(int, char *, DWORD))(*(_DWORD *)a1 + 4))(a1, (char *)v6[3] + v7, v4);
  }
  v6[0] = off_6D220;
  sub_620B0((int)v6);
  return v3;
}

