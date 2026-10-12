/* ea:   0x00059B9C */
/* name: sub_59B9C */
/* size: 100 */
void __fastcall sub_59B9C(int a1)
{
  unsigned int v2; // r5
  int v3; // r4

  if ( *(_BYTE *)(a1 + 68) != 0 )
  {
    EnterCriticalSection((LPCRITICAL_SECTION)(a1 + 40));
    v2 = 0;
    if ( *(_DWORD *)(a1 + 76) != 0 )
    {
      v3 = 0;
      do
      {
        sub_5AF80(v3 + *(_DWORD *)(a1 + 80));
        ++v2;
        v3 += 1304;
      }
      while ( v2 < *(_DWORD *)(a1 + 76) );
    }
    LeaveCriticalSection((LPCRITICAL_SECTION)(a1 + 40));
  }
}

