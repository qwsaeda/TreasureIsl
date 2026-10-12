/* ea:   0x0005879C */
/* name: sub_5879C */
/* size: 52 */
void __fastcall sub_5879C(int a1)
{
  void *v2; // r0

  if ( *(_BYTE *)(a1 + 28) != 0 )
  {
    v2 = *(void **)(a1 + 4);
    if ( v2 != nullptr )
    {
      operator delete(v2);
      *(_DWORD *)(a1 + 4) = 0;
      *(_BYTE *)(a1 + 28) = 0;
    }
  }
}

