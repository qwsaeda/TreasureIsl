/* ea:   0x0003FB1C */
/* name: sub_3FB1C */
/* size: 36 */
void __fastcall sub_3FB1C(int a1)
{
  void *v2; // r0

  v2 = *(void **)(a1 + 2088);
  if ( v2 != nullptr )
  {
    free(v2);
    *(_DWORD *)(a1 + 2088) = 0;
  }
}

