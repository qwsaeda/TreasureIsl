/* ea:   0x00063688 */
/* name: sub_63688 */
/* size: 96 */
int __fastcall sub_63688(int a1)
{
  int v2; // r0

  if ( *(_BYTE *)(a1 + 4) == 0 )
    __debugbreak();
  v2 = GXBeginDraw();
  if ( v2 != 0 )
    (*(void (__fastcall **)(int, _DWORD, int))(a1 + 48))(v2, *(_DWORD *)(a1 + 8), a1 + 12);
  return GXEndDraw();
}

