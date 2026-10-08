/* ea:   0x00063848 */
/* name: sub_63848 */
/* size: 208 */
int __fastcall sub_63848(int a1, HWND hWnd)
{
  int v4; // r5
  int v5; // r4
  int v6; // r0
  _DWORD *v7; // r0
  struct tagRECT prc; // [sp+0h] [bp-20h] BYREF

  if ( hWnd == nullptr )
    __debugbreak();
  if ( *(_BYTE *)(a1 + 4) != 0 )
    (*(void (__fastcall **)(int))(*(_DWORD *)a1 + 8))(a1);
  *(_DWORD *)(a1 + 12) = hWnd;
  if ( !IsWindow(hWnd) )
    __debugbreak();
  GetClientRect(*(HWND *)(a1 + 12), &prc);
  v4 = prc.right - prc.left;
  v5 = prc.bottom - prc.top;
  v6 = sub_63AE0(prc.right - prc.left, prc.bottom - prc.top);
  *(_DWORD *)(a1 + 16) = v6;
  if ( v6 == 0 )
    return 0;
  v7 = sub_62694(v4, v5, 16, 1);
  *(_DWORD *)(a1 + 8) = v7;
  if ( v7 == nullptr )
    return 0;
  *(_BYTE *)(a1 + 4) = 1;
  return 1;
}

