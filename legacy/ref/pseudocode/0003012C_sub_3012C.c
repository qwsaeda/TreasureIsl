/* ea:   0x0003012C */
/* name: sub_3012C */
/* size: 20 */
void __fastcall sub_3012C(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

