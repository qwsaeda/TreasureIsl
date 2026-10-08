/* ea:   0x0001E81C */
/* name: sub_1E81C */
/* size: 20 */
void __fastcall sub_1E81C(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

