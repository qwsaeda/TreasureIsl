/* ea:   0x00063918 */
/* name: sub_63918 */
/* size: 60 */
void __fastcall sub_63918(int a1)
{
  void *v2; // r0
  void *v3; // r0

  v2 = *(void **)(a1 + 16);
  if ( v2 != nullptr )
  {
    DeleteObject(v2);
    *(_DWORD *)(a1 + 16) = 0;
  }
  v3 = *(void **)(a1 + 8);
  if ( v3 != nullptr )
  {
    free(v3);
    *(_DWORD *)(a1 + 8) = 0;
  }
  *(_BYTE *)(a1 + 4) = 0;
}

