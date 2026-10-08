/* ea:   0x00062E68 */
/* name: sub_62E68 */
/* size: 152 */
int __fastcall sub_62E68(int a1)
{
  struct CRITICAL_SECTION *v2; // r5
  int v3; // r6
  _DWORD *v4; // r3
  _DWORD *v5; // r2
  int v6; // r0
  int v8; // r6

  v2 = (struct CRITICAL_SECTION *)(a1 + 48);
  v3 = 0;
  EnterCriticalSection((LPCRITICAL_SECTION)(a1 + 48));
  v4 = *(_DWORD **)(a1 + 92);
  if ( v4 != nullptr )
  {
    v5 = nullptr;
    while ( 1 )
    {
      v6 = v4[2];
      if ( v4[1] == MEMORY[0xFFFFC808] )
        break;
      v5 = v4;
      v4 = (_DWORD *)v4[2];
      if ( v6 == 0 )
      {
        LeaveCriticalSection(v2);
        return 0;
      }
    }
    if ( v5 == nullptr )
    {
      *(_DWORD *)(a1 + 92) = v6;
      v8 = *v4;
      LeaveCriticalSection(v2);
      return v8;
    }
    v5[2] = v6;
    v3 = *v4;
  }
  LeaveCriticalSection(v2);
  return v3;
}

