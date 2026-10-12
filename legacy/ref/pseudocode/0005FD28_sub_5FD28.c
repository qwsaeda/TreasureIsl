/* ea:   0x0005FD28 */
/* name: sub_5FD28 */
/* size: 488 */
int __fastcall sub_5FD28(int a1, _WORD *a2)
{
  int v4; // r0
  int v6; // r4
  _WORD *v7; // [sp+8h] [bp-48h] BYREF
  _BYTE *v8; // [sp+Ch] [bp-44h]
  _BYTE v9[4]; // [sp+18h] [bp-38h] BYREF
  _BYTE v10[12]; // [sp+1Ch] [bp-34h] BYREF
  int v11; // [sp+28h] [bp-28h]
  _BYTE v12[32]; // [sp+30h] [bp-20h] BYREF

  if ( a2 == nullptr )
    __debugbreak();
  sub_3E0A4(&v7);
  v4 = sub_12F04(a2);
  sub_3E0B8(&v7, a2, &a2[v4]);
  sub_60628(a1, &v7);
  sub_60690(a1, &v7);
  sub_6070C(a1, &v7);
  if ( v7 == (_WORD *)v8 )
    goto LABEL_8;
  if ( *v7 == 91 && v7[((v8 - (_BYTE *)v7) >> 1) - 1] == 93 )
  {
    sub_13110((int *)&v7, 0, 1);
    sub_13110((int *)&v7, ((v8 - (_BYTE *)v7) >> 1) - 1, -1);
    sub_5FF10(a1, v9, &v7);
    *(_DWORD *)(a1 + 16) = v11;
    sub_12EA0((int)v10);
    sub_12EA0((int)&v7);
    return 1;
  }
  v6 = sub_607B0(&v7, 61, 0);
  if ( v6 == -1 )
  {
LABEL_8:
    sub_12F48((void **)&v7);
    return 1;
  }
  else if ( *(_DWORD *)(a1 + 16) != 0 )
  {
    sub_60098(&v7, v12, 0, v6);
    sub_60690(a1, v12);
    sub_60098(&v7, v9, v6 + 1, -1);
    sub_60690(a1, v9);
    sub_608C4(a1, *(_DWORD *)(a1 + 16), v12, v9);
    sub_12EA0((int)v9);
    sub_12EA0((int)v12);
    sub_12EA0((int)&v7);
    return 1;
  }
  else
  {
    sub_12F48((void **)&v7);
    return 0;
  }
}

