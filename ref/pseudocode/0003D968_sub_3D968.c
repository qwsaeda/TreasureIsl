/* ea:   0x0003D968 */
/* name: sub_3D968 */
/* size: 452 */
int __fastcall sub_3D968(int a1)
{
  int v2; // r1
  void (__fastcall ***v3)(_DWORD, int); // r3
  int v4; // r1
  void (__fastcall ***v5)(_DWORD, int); // r3
  void *v6; // r4
  int v7; // r0
  void *v8; // r4
  void *v9; // r4

  v2 = *(_DWORD *)(a1 + 8);
  if ( v2 != 0 )
  {
    (*(void (__fastcall **)(_DWORD))(*(_DWORD *)v2 + 12))(*(_DWORD *)(a1 + 8));
    v3 = *(void (__fastcall ****)(_DWORD, int))(a1 + 8);
    if ( v3 != nullptr )
      (**v3)(*(_DWORD *)(a1 + 8), 1);
    *(_DWORD *)(a1 + 8) = 0;
  }
  if ( *(_DWORD *)(a1 + 4) != 0 )
  {
    (***(void (__fastcall ****)(_DWORD, int))(a1 + 4))(*(_DWORD *)(a1 + 4), 1);
    *(_DWORD *)(a1 + 4) = 0;
  }
  v4 = *(_DWORD *)(a1 + 16);
  if ( v4 != 0 )
  {
    (*(void (__fastcall **)(_DWORD))(*(_DWORD *)v4 + 8))(*(_DWORD *)(a1 + 16));
    sub_43C5C(*(_DWORD *)(a1 + 16));
    v5 = *(void (__fastcall ****)(_DWORD, int))(a1 + 16);
    if ( v5 != nullptr )
      (**v5)(*(_DWORD *)(a1 + 16), 1);
    *(_DWORD *)(a1 + 16) = 0;
  }
  v6 = *(void **)(a1 + 24);
  if ( v6 != nullptr )
  {
    sub_3E520(*(_DWORD *)(a1 + 24), v4);
    operator delete(v6);
    *(_DWORD *)(a1 + 24) = 0;
  }
  if ( *(_DWORD *)(a1 + 28) != 0 )
  {
    *(_BYTE *)(a1 + 32) = 0;
    sub_3E8CC();
    v7 = *(_DWORD *)(a1 + 28);
    *(_BYTE *)(a1 + 33) = 0;
    sub_3E940(v7, aMusicPressWav);
    sub_3E940(*(_DWORD *)(a1 + 28), aMusicInvalidWa);
    sub_3E940(*(_DWORD *)(a1 + 28), aMusicAttackWav);
    sub_3E940(*(_DWORD *)(a1 + 28), aMusicLevelupWa);
    sub_3E940(*(_DWORD *)(a1 + 28), aMusicVictoryWa);
    v8 = *(void **)(a1 + 28);
    if ( v8 != nullptr )
    {
      sub_3DB2C(*(_DWORD *)(a1 + 28));
      operator delete(v8);
    }
    *(_DWORD *)(a1 + 28) = 0;
  }
  if ( *(_DWORD *)(a1 + 20) != 0 )
  {
    (***(void (__fastcall ****)(_DWORD, int))(a1 + 20))(*(_DWORD *)(a1 + 20), 1);
    *(_DWORD *)(a1 + 20) = 0;
  }
  v9 = *(void **)(a1 + 60);
  if ( v9 != nullptr )
  {
    sub_42A34(*(_DWORD *)(a1 + 60));
    operator delete(v9);
    *(_DWORD *)(a1 + 60) = 0;
  }
  return 1;
}

