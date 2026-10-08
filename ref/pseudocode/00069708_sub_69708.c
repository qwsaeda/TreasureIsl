/* ea:   0x00069708 */
/* name: sub_69708 */
/* size: 576 */
int __fastcall sub_69708(
        unsigned int a1,
        unsigned int a2,
        int *a3,
        unsigned int *a4,
        unsigned int *a5,
        _DWORD *a6,
        _DWORD *a7,
        int a8,
        int a9)
{
  unsigned int *v13; // r0
  unsigned int *v14; // r10
  int v16; // r0
  int v17; // r9
  int v18; // r0
  int v19; // r5
  void (__fastcall *v20)(_DWORD, unsigned int *); // r2
  void (__fastcall *v21)(_DWORD, unsigned int *); // r2
  void (__fastcall *v22)(_DWORD, unsigned int *); // r2
  unsigned int v23[10]; // [sp+18h] [bp-28h] BYREF

  v23[0] = 0;
  v13 = (unsigned int *)(*(int (__fastcall **)(_DWORD, int, int))(a9 + 32))(*(_DWORD *)(a9 + 40), 288, 4);
  v14 = v13;
  if ( v13 == nullptr )
    return -4;
  v16 = sub_690F0(a3, a1, 0x101u, (int)&unk_7BDF0, (int)&unk_7BE70, a6, a4, a8, v23, v13);
  v17 = v16;
  if ( v16 == 0 )
  {
    if ( *a4 != 0 )
    {
      v18 = sub_690F0(&a3[a1], a2, 0, (int)&unk_7BEF0, (int)&unk_7BF68, a7, a5, a8, v23, v14);
      v19 = v18;
      if ( v18 != 0 )
      {
        switch ( v18 )
        {
          case -3:
            v20 = *(void (__fastcall **)(_DWORD, unsigned int *))(a9 + 36);
            *(_DWORD *)(a9 + 24) = aOversubscribed_0;
            v20(*(_DWORD *)(a9 + 40), v14);
            return -3;
          case -5:
            v21 = *(void (__fastcall **)(_DWORD, unsigned int *))(a9 + 36);
            *(_DWORD *)(a9 + 24) = aIncompleteDist;
            v21(*(_DWORD *)(a9 + 40), v14);
            return -3;
          case -4:
            goto LABEL_15;
          default:
            break;
        }
      }
      else if ( *a5 != 0 || a1 <= 0x101 )
      {
        (*(void (__fastcall **)(_DWORD, unsigned int *))(a9 + 36))(*(_DWORD *)(a9 + 40), v14);
        return 0;
      }
      v19 = -3;
      *(_DWORD *)(a9 + 24) = aEmptyDistanceT;
LABEL_15:
      (*(void (__fastcall **)(_DWORD, unsigned int *))(a9 + 36))(*(_DWORD *)(a9 + 40), v14);
      return v19;
    }
LABEL_19:
    v17 = -3;
    *(_DWORD *)(a9 + 24) = aIncompleteLite;
    goto LABEL_20;
  }
  if ( v16 == -3 )
  {
    v22 = *(void (__fastcall **)(_DWORD, unsigned int *))(a9 + 36);
    *(_DWORD *)(a9 + 24) = aOversubscribed_1;
    v22(*(_DWORD *)(a9 + 40), v14);
    return -3;
  }
  if ( v16 != -4 )
    goto LABEL_19;
LABEL_20:
  (*(void (__fastcall **)(_DWORD, unsigned int *))(a9 + 36))(*(_DWORD *)(a9 + 40), v14);
  return v17;
}

