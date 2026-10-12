/* ea:   0x0004B22C */
/* name: sub_4B22C */
/* size: 412 */
void __fastcall sub_4B22C(_DWORD *a1, int a2, int a3)
{
  int v5; // r0
  int v6; // r0
  int v7; // r0
  bool v8; // zf
  int v9; // r0
  int v10; // r4
  char v11; // r5
  void *v12[4]; // [sp+0h] [bp-2Ch] BYREF
  void *v13[7]; // [sp+10h] [bp-1Ch] BYREF

  if ( a2 == 0 )
  {
    sub_1333C(v13);
    sub_1333C(v12);
    switch ( a3 )
    {
      case 13:
        v7 = a1[255];
        if ( v7 != 0 )
        {
          if ( v7 == 1 )
          {
            v8 = a1[257] == 0;
            v9 = a1[258];
            a1[257] = v8;
            *(_BYTE *)(v9 + 33) = v8;
            sub_445F4((int)a1, 2)[5] = a1[257];
            sub_12F48(v12);
            sub_12F48(v13);
            return;
          }
          if ( v7 == 2 )
          {
            a1[252] = 1;
            sub_12F48(v12);
            sub_12F48(v13);
            return;
          }
        }
        else
        {
          v10 = a1[258];
          if ( a1[256] != 0 )
          {
            v11 = 0;
            a1[256] = 0;
            sub_59B9C(*(_DWORD *)(*(_DWORD *)(v10 + 28) + 20));
          }
          else
          {
            v11 = 1;
            a1[256] = 1;
            sub_3E758(*(_DWORD *)(v10 + 28), *(_WORD **)(v10 + 36));
          }
          *(_BYTE *)(v10 + 32) = v11;
          sub_445F4((int)a1, 1)[5] = a1[256];
        }
        break;
      case 38:
        v6 = a1[255];
        a1[255] = v6 - 1;
        if ( v6 - 1 < 0 )
        {
          a1[255] = 2;
          sub_12F48(v12);
          sub_12F48(v13);
          return;
        }
        break;
      case 40:
        v5 = a1[255];
        a1[255] = v5 + 1;
        if ( v5 + 1 > 2 )
        {
          a1[255] = 0;
          sub_12F48(v12);
          sub_12F48(v13);
          return;
        }
        break;
      default:
        break;
    }
    sub_12F48(v12);
    sub_12F48(v13);
  }
}

