/* ea:   0x00063954 */
/* name: sub_63954 */
/* size: 252 */
HDC __fastcall sub_63954(int a1)
{
  int v2; // r1
  HDC result; // r0
  HDC v4; // r6
  HDC CompatibleDC; // r5
  HGDIOBJ v6; // r0
  int v7; // r1
  void *v8; // r4
  _BYTE v9[20]; // [sp+14h] [bp-2Ch] BYREF
  void *v10; // [sp+28h] [bp-18h]

  if ( *(_BYTE *)(a1 + 4) == 0 )
    __debugbreak();
  GetObjectW(*(HGDIOBJ *)(a1 + 16), 24, v9);
  v2 = *(_DWORD *)(a1 + 8);
  if ( v2 == 0 )
  {
    __debugbreak();
    __debugbreak();
  }
  memcpy(v10, *(const void **)(v2 + 28), *(_DWORD *)(*(_DWORD *)(a1 + 8) + 16));
  result = GetDC(*(HWND *)(a1 + 12));
  v4 = result;
  if ( result != nullptr )
  {
    CompatibleDC = CreateCompatibleDC(result);
    v6 = SelectObject(CompatibleDC, *(HGDIOBJ *)(a1 + 16));
    v7 = *(_DWORD *)(a1 + 8);
    v8 = v6;
    if ( v7 == 0 )
    {
      __debugbreak();
      __debugbreak();
    }
    BitBlt(v4, 0, 0, *(_DWORD *)(v7 + 4), *(_DWORD *)(*(_DWORD *)(a1 + 8) + 8), CompatibleDC, 0, 0, 0xCC0020u);
    SelectObject(CompatibleDC, v8);
    DeleteDC(CompatibleDC);
    return (HDC)ReleaseDC(*(HWND *)(a1 + 12), v4);
  }
  return result;
}

