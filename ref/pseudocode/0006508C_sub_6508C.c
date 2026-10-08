/* ea:   0x0006508C */
/* name: sub_6508C */
/* size: 52 */
void __fastcall sub_6508C(int a1)
{
  void *v2; // r0

  if ( *(_BYTE *)(a1 + 12) != 0 )
  {
    v2 = *(void **)(a1 + 16);
    if ( v2 != nullptr )
    {
      operator delete(v2);
      *(_DWORD *)(a1 + 16) = 0;
    }
    *(_BYTE *)(a1 + 12) = 0;
  }
}

