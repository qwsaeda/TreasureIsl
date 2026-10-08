/* ea:   0x00028B48 */
/* name: sub_28B48 */
/* size: 1392 */
int __fastcall sub_28B48(int a1)
{
  void **i; // r4
  void **j; // r4
  _DWORD *k; // r5
  void (__fastcall ***v5)(_DWORD, int); // r3
  _DWORD *m; // r6
  void *v7; // r4
  _DWORD *n; // r5
  _DWORD *v9; // r4
  _DWORD *ii; // r5
  _DWORD *v11; // r4
  _DWORD *jj; // r6
  void *v13; // r4
  int v14; // r3
  void (__fastcall ***v15)(_DWORD, int); // r3
  void (__fastcall ***v16)(_DWORD, int); // r3
  void (__fastcall ***v17)(_DWORD, int); // r3
  void (__fastcall ***v18)(_DWORD, int); // r3
  int v20; // [sp+0h] [bp-2Ch] BYREF
  int v21; // [sp+4h] [bp-28h] BYREF
  _DWORD v22[9]; // [sp+8h] [bp-24h] BYREF

  sub_3A650(a1 + 192);
  if ( *(_DWORD *)(a1 + 44) != 0 )
  {
    operator delete(*(void **)(a1 + 44));
    *(_DWORD *)(a1 + 44) = 0;
  }
  for ( i = *(void ***)(a1 + 20); i != *(void ***)(a1 + 24); ++i )
  {
    if ( *i != nullptr )
    {
      free(*i);
      *i = nullptr;
    }
  }
  sub_1DE28(a1 + 20, *(char **)(a1 + 20), *(_BYTE **)(a1 + 24));
  for ( j = *(void ***)(a1 + 32); j != *(void ***)(a1 + 36); ++j )
  {
    if ( *j != nullptr )
    {
      free(*j);
      *j = nullptr;
    }
  }
  sub_1DE28(a1 + 32, *(char **)(a1 + 32), *(_BYTE **)(a1 + 36));
  sub_33CC4(a1 + 72);
  (*(void (__fastcall **)(int))(*(_DWORD *)(a1 + 288) + 16))(a1 + 288);
  v20 = a1 + 288;
  sub_2F7B0(a1 + 64, &v20);
  for ( k = *(_DWORD **)sub_30C2C(a1 + 496, v22); k != (_DWORD *)*sub_1DE18((_DWORD *)(a1 + 496), v22); k = sub_16A30(k) )
  {
    if ( k[5] != 0 )
    {
      (*(void (__fastcall **)(_DWORD))(*(_DWORD *)k[5] + 16))(k[5]);
      v21 = k[5];
      sub_2F7B0(a1 + 64, &v21);
      v5 = (void (__fastcall ***)(_DWORD, int))k[5];
      if ( v5 != nullptr )
        (**v5)(k[5], 1);
      k[5] = 0;
    }
  }
  sub_30CD8(a1 + 496);
  for ( m = *(_DWORD **)(*(_DWORD *)(a1 + 520) + 8); m != *(_DWORD **)(a1 + 520); m = sub_16A30(m) )
  {
    v7 = (void *)m[5];
    if ( v7 != nullptr )
    {
      sub_1B2AC((_DWORD *)m[5]);
      operator delete(v7);
      m[5] = 0;
    }
  }
  sub_30E08(a1 + 520);
  for ( n = **(_DWORD ***)(a1 + 64); n != *(_DWORD **)(a1 + 64); n = (_DWORD *)*n )
  {
    v9 = n + 2;
    if ( n[2] != 0 )
    {
      (*(void (__fastcall **)(_DWORD))(*(_DWORD *)n[2] + 16))(n[2]);
      if ( *v9 != 0 )
        (**(void (__fastcall ***)(_DWORD, int))*v9)(*v9, 1);
      *v9 = 0;
    }
  }
  sub_36E60(a1 + 64);
  for ( ii = **(_DWORD ***)(a1 + 68); ii != *(_DWORD **)(a1 + 68); ii = (_DWORD *)*ii )
  {
    v11 = ii + 2;
    if ( ii[2] != 0 )
    {
      (*(void (__fastcall **)(_DWORD))(*(_DWORD *)ii[2] + 16))(ii[2]);
      if ( *v11 != 0 )
        (**(void (__fastcall ***)(_DWORD, int))*v11)(*v11, 1);
      *v11 = 0;
    }
  }
  sub_36E60(a1 + 68);
  for ( jj = *(_DWORD **)(*(_DWORD *)(a1 + 508) + 8); jj != *(_DWORD **)(a1 + 508); jj = sub_16A30(jj) )
  {
    if ( jj[5] != 0 )
    {
      sub_44C68();
      v13 = (void *)jj[5];
      if ( v13 != nullptr )
      {
        sub_44B68(jj[5]);
        operator delete(v13);
      }
      jj[5] = 0;
    }
  }
  sub_303B8(a1 + 508);
  sub_53714(a1 + 52);
  v14 = *(_DWORD *)(a1 + 580);
  if ( v14 != 0 )
  {
    (*(void (__fastcall **)(_DWORD))(*(_DWORD *)v14 + 8))(*(_DWORD *)(a1 + 580));
    v15 = *(void (__fastcall ****)(_DWORD, int))(a1 + 580);
    if ( v15 != nullptr )
      (**v15)(*(_DWORD *)(a1 + 580), 1);
    *(_DWORD *)(a1 + 580) = 0;
  }
  if ( *(_DWORD *)(a1 + 576) != 0 )
  {
    (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 576) + 8))(*(_DWORD *)(a1 + 576));
    v16 = *(void (__fastcall ****)(_DWORD, int))(a1 + 576);
    if ( v16 != nullptr )
      (**v16)(*(_DWORD *)(a1 + 576), 1);
    *(_DWORD *)(a1 + 576) = 0;
  }
  if ( *(_DWORD *)(a1 + 584) != 0 )
  {
    (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 584) + 8))(*(_DWORD *)(a1 + 584));
    v17 = *(void (__fastcall ****)(_DWORD, int))(a1 + 584);
    if ( v17 != nullptr )
      (**v17)(*(_DWORD *)(a1 + 584), 1);
    *(_DWORD *)(a1 + 584) = 0;
  }
  if ( *(_DWORD *)(a1 + 596) != 0 )
  {
    (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 596) + 12))(*(_DWORD *)(a1 + 596));
    v18 = *(void (__fastcall ****)(_DWORD, int))(a1 + 596);
    if ( v18 != nullptr )
      (**v18)(*(_DWORD *)(a1 + 596), 1);
    *(_DWORD *)(a1 + 596) = 0;
  }
  if ( *(_DWORD *)(a1 + 588) != 0 )
  {
    (***(void (__fastcall ****)(_DWORD, int))(a1 + 588))(*(_DWORD *)(a1 + 588), 1);
    *(_DWORD *)(a1 + 588) = 0;
  }
  if ( *(_DWORD *)(a1 + 592) != 0 )
  {
    (***(void (__fastcall ****)(_DWORD, int))(a1 + 592))(*(_DWORD *)(a1 + 592), 1);
    *(_DWORD *)(a1 + 592) = 0;
  }
  *(_DWORD *)(a1 + 732) = 0;
  *(_DWORD *)(a1 + 600) = 0;
  *(_DWORD *)(a1 + 604) = 0;
  *(_DWORD *)(a1 + 688) = 0;
  sub_310E4(a1 + 552);
  sub_310E4(a1 + 564);
  MEMORY[0x10008113C] = 0;
  MEMORY[0x100081140] = 0;
  MEMORY[0x100081144] = 0;
  MEMORY[0x100081148] = 0;
  MEMORY[0x10008114C] = 0;
  MEMORY[0x100081150] = 0;
  MEMORY[0x100081154] = 0;
  MEMORY[0x100081158] = 0;
  MEMORY[0x10008115C] = 0;
  MEMORY[0x100081160] = 0;
  MEMORY[0x100081164] = 0;
  MEMORY[0x100081168] = 0;
  MEMORY[0x10008116C] = 0;
  MEMORY[0x100081170] = 0;
  MEMORY[0x100081174] = 0;
  word_81178 = 0;
  off_81138 = nullptr;
  dword_8117C = 0;
  memset(word_81038, 0, sizeof(word_81038));
  return 1;
}

